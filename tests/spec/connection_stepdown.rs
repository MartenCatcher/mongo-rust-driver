use std::sync::Mutex;

use assert_matches::assert_matches;
use bson::{bson, doc};
use lazy_static::lazy_static;
use mongodb::{
    error::{CommandError, ErrorKind},
    options::{
        Acknowledgment, CreateCollectionOptions, DropCollectionOptions, FindOptions,
        InsertManyOptions, WriteConcern,
    },
    Collection, Database,
};

use crate::util::EventClient;

lazy_static! {
    // Ensures that only one of the stepdown tests is running at a time. This is necessary due to the use of failpoints.
    static ref STEPDOWN_TEST_MUTEX: Mutex<()> = Mutex::new(());
}

fn run_test(name: &str, test: impl Fn(EventClient, Database, Collection)) {
    // TODO RUST-51: Disable retryable writes once they're implemented.
    let client = EventClient::new();

    if client.options.repl_set_name.is_none() {
        return;
    }

    let name = format!("step-down-{}", name);

    let db = client.database(&name);
    let coll = db.collection(&name);

    let wc_majority = WriteConcern::builder().w(Acknowledgment::Majority).build();

    let _ = coll.drop(Some(
        DropCollectionOptions::builder()
            .write_concern(wc_majority.clone())
            .build(),
    ));

    db.create_collection(
        &name,
        Some(
            CreateCollectionOptions::builder()
                .write_concern(wc_majority.clone())
                .build(),
        ),
    )
    .unwrap();

    test(client, db, coll);
}

#[function_name::named]
#[test]
fn get_more() {
    run_test(function_name!(), |client, db, coll| {
        let _lock = STEPDOWN_TEST_MUTEX.lock();

        // This test requires server version 4.2 or higher.
        if client.server_version_lt(4, 2) {
            return;
        }

        let docs = vec![doc! { "x": 1 }; 5];
        coll.insert_many(
            docs,
            Some(
                InsertManyOptions::builder()
                    .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
                    .build(),
            ),
        )
        .unwrap();

        let mut cursor = coll
            .find(None, Some(FindOptions::builder().batch_size(2).build()))
            .unwrap();

        db.run_command(doc! { "replSetStepDown": 5, "force": true }, None)
            .expect("stepdown should have succeeded");

        for _ in 0..5 {
            cursor
                .next()
                .unwrap()
                .expect("cursor iteration should have succeeded");
        }

        assert!(client.pool_cleared_events.read().unwrap().is_empty());
    });
}

#[function_name::named]
#[test]
fn not_master_keep_pool() {
    run_test(function_name!(), |client, _, coll| {
        // This test requires server version 4.2 or higher.
        if client.server_version_lt(4, 2) {
            return;
        }

        let _lock = STEPDOWN_TEST_MUTEX.lock();

        client
            .database("admin")
            .run_command(
                doc! {
                    "configureFailPoint": "failCommand",
                    "mode": { "times": 1 },
                    "data": {
                        "failCommands": ["insert"],
                        "errorCode": 10107
                    }
                },
                None,
            )
            .unwrap();

        let result = coll.insert_one(doc! { "test": 1 }, None);
        assert_matches!(
            result.as_ref().map_err(|e| e.as_ref()),
            Err(ErrorKind::CommandError(CommandError { code: 10107, .. })),
            "insert should have failed"
        );

        coll.insert_one(doc! { "test": 1 }, None)
            .expect("insert should have succeeded");

        assert!(client.pool_cleared_events.read().unwrap().is_empty());
    });
}

#[function_name::named]
#[test]
fn not_master_reset_pool() {
    run_test(function_name!(), |client, _, coll| {
        // This test must only run on 4.0 servers.
        if !client.server_version_eq(4, 0) {
            return;
        }

        let _lock = STEPDOWN_TEST_MUTEX.lock();

        client
            .database("admin")
            .run_command(
                doc! {
                    "configureFailPoint": "failCommand",
                    "mode": { "times": 1 },
                    "data": {
                        "failCommands": ["insert"],
                        "errorCode": 10107
                    }
                },
                None,
            )
            .unwrap();

        let result = coll.insert_one(doc! { "test": 1 }, None);
        assert_matches!(
            result.as_ref().map_err(|e| e.as_ref()),
            Err(ErrorKind::CommandError(CommandError { code: 10107, .. })),
            "insert should have failed"
        );

        assert!(client.pool_cleared_events.read().unwrap().len() == 1);

        coll.insert_one(doc! { "test": 1 }, None)
            .expect("insert should have succeeded");
    });
}

#[function_name::named]
#[test]
fn shutdown_in_progress() {
    run_test(function_name!(), |client, _, coll| {
        if client.server_version_lt(4, 0) {
            return;
        }

        let _lock = STEPDOWN_TEST_MUTEX.lock();

        client
            .database("admin")
            .run_command(
                doc! {
                    "configureFailPoint": "failCommand",
                    "mode": { "times": 1 },
                    "data": {
                        "failCommands": ["insert"],
                        "errorCode": 91
                    }
                },
                None,
            )
            .unwrap();

        let result = coll.insert_one(doc! { "test": 1 }, None);
        assert_matches!(
            result.as_ref().map_err(|e| e.as_ref()),
            Err(ErrorKind::CommandError(CommandError { code: 91, .. })),
            "insert should have failed"
        );

        assert!(client.pool_cleared_events.read().unwrap().len() == 1);

        coll.insert_one(doc! { "test": 1 }, None)
            .expect("insert should have succeeded");
    })
}

#[function_name::named]
#[test]
fn interrupted_at_shutdown() {
    run_test(function_name!(), |client, _, coll| {
        if client.server_version_lt(4, 0) {
            return;
        }

        let _lock = STEPDOWN_TEST_MUTEX.lock();

        client
            .database("admin")
            .run_command(
                doc! {
                    "configureFailPoint": "failCommand",
                    "mode": { "times": 1 },
                    "data": {
                        "failCommands": ["insert"],
                        "errorCode": 11600
                    }
                },
                None,
            )
            .unwrap();

        let result = coll.insert_one(doc! { "test": 1 }, None);
        assert_matches!(
            result.as_ref().map_err(|e| e.as_ref()),
            Err(ErrorKind::CommandError(CommandError { code: 11600, .. })),
            "insert should have failed"
        );

        assert!(client.pool_cleared_events.read().unwrap().len() == 1);

        coll.insert_one(doc! { "test": 1 }, None)
            .expect("insert should have succeeded");
    })
}
