use ucel_testkit::fuzz::{mutate_bytes, ws_guard_entry, XorShift64};
use ucel_testkit::fuzz_corpus::{load_ws_frame_corpus, CorpusConfig};

const ITERATIONS: usize = 200;
const RNG_SEED: u64 = 0xC0DEC0FFEE;
const MAX_GENERATED_LEN: usize = 512 * 1024;

#[test]
fn fuzz_ws_frames_crash_free_and_bounded() {
    let corpus = load_ws_frame_corpus(CorpusConfig::default()).expect("ws corpus must load");
    let mut rng = XorShift64::new(RNG_SEED);

    for i in 0..ITERATIONS {
        let base = &corpus[rng.next_usize(corpus.len())].bytes;
        let mut input = mutate_bytes(base, &mut rng, MAX_GENERATED_LEN);

        if i % 50 == 0 {
            // generate oversized payload from a small seed without storing large fixture files
            let grow = if base.is_empty() {
                b"x".as_slice()
            } else {
                base.as_slice()
            };
            while input.len() <= MAX_GENERATED_LEN {
                input.extend_from_slice(grow);
                if grow.is_empty() {
                    break;
                }
            }
        }

        let result = ws_guard_entry(&input);
        if input.len() > MAX_GENERATED_LEN {
            assert!(result.is_err(), "oversized input must return Err");
        }
    }
}
