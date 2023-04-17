use serial_test::{parallel, serial};
use xtask::challange::{run, Challange, RunOptions};

fn run_challange(challange: Challange) {
    std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).unwrap();
    run(<RunOptions as clap::Parser>::parse_from([
        "--challenge",
        <Challange as clap::ValueEnum>::to_possible_value(&challange)
            .unwrap()
            .get_name(),
        "--release",
    ]));
}

#[test]
#[parallel]
fn run_echo() {
    run_challange(Challange::Echo);
}

#[test]
#[parallel]
fn run_unique() {
    run_challange(Challange::UniqueIds);
}

#[test]
#[parallel]
fn run_single_broadcast() {
    run_challange(Challange::SingleBroadcast);
}

#[test]
#[parallel]
fn run_multi_broadcast() {
    run_challange(Challange::MultiBroadcast);
}

#[test]
#[parallel]
fn run_faulty_broadcast() {
    run_challange(Challange::FaultyBroadcast);
}

#[test]
#[serial]
fn run_efficent_broadcast() {
    run_challange(Challange::EfficientBroadcast);
}

#[test]
#[serial]
fn run_efficent_broadcast2() {
    run_challange(Challange::EfficientBroadcast2);
}

#[test]
#[parallel]
fn run_grow_only_counter() {
    run_challange(Challange::GrowOnlyCounter);
}
