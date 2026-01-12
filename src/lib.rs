#![feature(test)]
extern crate test;

#[cfg(not(test))]
use log::debug;
#[cfg(test)]
use std::println as debug;

use std::char;

use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub fn levenshtein_distance(s: &str, t: &str) -> usize {
    levenshtein_distance_chars(
        &s.chars().collect::<Vec<_>>(),
        &t.chars().collect::<Vec<_>>(),
    )
}

fn levenshtein_distance_chars(s: &[char], t: &[char]) -> usize {
    let m = s.len();
    let n = t.len();
    let mut d = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        d[i][0] = i;
    }

    for j in 1..=n {
        d[0][j] = j;
    }

    for j in 1..=n {
        for i in 1..=m {
            let substitution_cost = if s[i - 1] == t[j - 1] { 0 } else { 1 };

            d[i][j] = (d[i - 1][j] + 1)
                .min(d[i][j - 1] + 1)
                .min(d[i - 1][j - 1] + substitution_cost);
        }
    }

    d[m][n]
}

fn normalize(s: &str) -> Vec<char> {
    let mut normalized = vec![];

    for unicode_char in s.graphemes(true) {
        for part in unicode_char.chars() {
            for char in part.nfd() {
                if char == ' ' {
                    continue;
                }
                normalized.push(match char {
                    'ㄱ' | 'ㅋ' | 'ㄲ' => 'ㄱ',
                    'ㄷ' | 'ㄸ' | 'ㅌ' => 'ㄷ',
                    'ㅂ' | 'ㅃ' | 'ㅍ' => 'ㅂ',
                    'ㅅ' | 'ㅆ' => 'ㅅ',
                    'ㅈ' | 'ㅉ' | 'ㅊ' => 'ㅈ',
                    _ => char,
                });
            }
        }
    }

    normalized
}

/// Returns 1.0 for most different and 0.0 for exactly the same.
/// Implementation of "Word Similarity Calculation by Using the Edit Distance Metrics with Consonant Normalization" https://web.archive.org/web/20260112025218/https://koreascience.kr/article/JAKO201502152089381.pdf from Kang Seung Shik I'm not sure if it's 100% correct.
///
/// # Examples
///
/// ```
/// let distance = k_edit_distance::k_edit_distance("국어", "숙어");
/// assert_eq!(distance, 0.16666667);
/// let distance = k_edit_distance::k_edit_distance("신문", "신문");
/// assert_eq!(distance, 0.0);
/// let distance = k_edit_distance::k_edit_distance("하늘", "택시");
/// assert_eq!(distance, 1.0);
/// ```
pub fn k_edit_distance(s: &str, t: &str) -> f32 {
    if s.len() == 0 && t.len() == 0 {
        return 0.;
    }
    debug!("{} to {}", s, t);

    // break each string into syllables
    let s_syllables: Vec<_> = s.graphemes(true).collect();
    let t_syllables: Vec<_> = t.graphemes(true).collect();

    let mut edit_distance = 0;
    for i in 0..(s_syllables.len().max(t_syllables.len())) {
        let s_part = s_syllables.get(i).unwrap_or(&"");
        let s_norm = normalize(s_part);
        let t_part = t_syllables.get(i).unwrap_or(&"");
        let t_norm = normalize(t_part);

        let syllable_dist = levenshtein_distance_chars(&s_norm, &t_norm);

        edit_distance += syllable_dist;
        debug!(
            "{} {}({}) ({}({:?}) {}({:?}))",
            i, edit_distance, syllable_dist, s_part, s_norm, t_part, t_norm
        );
    }

    let max = (3 * s_syllables.len()).max(3 * t_syllables.len());
    let n = edit_distance as f32 / max as f32;
    debug!("{} / {} = {}", edit_distance, max, n);

    n
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use test::Bencher;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("kitten", "sitten"), 1);
        assert_eq!(levenshtein_distance("sitten", "sittin"), 1);
        assert_eq!(levenshtein_distance("sittin", "sitting"), 1);
        assert_eq!(levenshtein_distance("book", "back"), 2);
        assert_eq!(levenshtein_distance("apple", "back"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
    }

    #[test]
    fn test_kang_seung_shik_distance() {
        // These are from the paper
        assert_eq!(k_edit_distance("국어", "숙어"), 0.16666667);
        assert_eq!(k_edit_distance("나무가지", "나뭇가지"), 0.083333336);
        // These are ones that confuse me
        assert_eq!(k_edit_distance("신문", "식물"), 0.33333334);
        assert_eq!(k_edit_distance("신문", "식물"), 0.33333334);
        // These are super different
        assert_eq!(k_edit_distance("검은색", "분홍색"), 0.6666667);
        assert_eq!(k_edit_distance("신호등", "택시"), 0.8888889);
        assert_eq!(k_edit_distance("진공청소기", "솥"), 0.8666667);
        assert_eq!(k_edit_distance("하늘", "택시"), 1.0);

        assert_eq!(k_edit_distance("", ""), 0.);
    }

    #[bench]
    fn bench_add_two(b: &mut Bencher) {
        const WORDS: &[&str] = &[
            "국어",
            "숙어",
            "나무가지",
            "신문",
            "검은색",
            "분홍색",
            "진공청소기",
            "택시",
            "모시금자라남생이잎벌레",
        ];
        b.iter(|| {
            for outer in WORDS {
                for inner in WORDS {
                    k_edit_distance(inner, outer);
                }
            }
        });
    }
}
