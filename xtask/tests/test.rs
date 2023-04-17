use rstest::*;
use xtask::challange::{run, Challange, RunOptions};

#[rstest]
#[case(Challange::Echo)]
#[case(Challange::UniqueIds)]
#[case(Challange::SingleBroadcast)]
#[case(Challange::MultiBroadcast)]
#[case(Challange::FaultyBroadcast)]
#[case(Challange::EfficientBroadcast)]
#[case(Challange::EfficientBroadcast2)]
#[case(Challange::GrowOnlyCounter)]
fn run_challange(#[case] challange: Challange) {
    std::env::set_current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/..")).unwrap();
    run(<RunOptions as clap::Parser>::parse_from([
        "--challenge",
        <Challange as clap::ValueEnum>::to_possible_value(&challange)
            .unwrap()
            .get_name(),
        "--release",
    ]));
}
