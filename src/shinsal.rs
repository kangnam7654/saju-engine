//! 신살(神殺) — 지지 삼합 + 갑자 매칭 기반 특수 작용력 판정.
//!
//! v0.2.0(saju-engine)에서는 7종만 우선 판정:
//! - 12신살 5종: 도화살(=년살), 역마살, 장성살, 반안살, 화개살
//! - 백호살(白虎殺) — 7갑자 매칭형
//! - 천을귀인(天乙貴人) — 일간별 두 지지
//!
//! 통변 톤은 Gemini 사주 보고서 §6의 "현대적 재해석" 라인을 그대로 흡수.
//! "흉살 = 단정"이 아니라 "현대 직업·매력 자본"으로 읽는다.
//!
//! 누락된 신살(겁살·재살·천살·지살·월살·망신살·육해살)은 v0.2.x 후속.
//!
//! ## 알고리즘 메모
//!
//! **12신살**은 **년지(年支)** 가 속한 삼합(三合)의 첫 글자에서 -3한 위치를
//! 겁살의 기점으로 잡고, 12지지를 정방향으로 0..11에 매핑한다.
//!
//! - 신자진(水) → 겁살 사 → 도화 유, 역마 인, 장성 자, 반안 축, 화개 진
//! - 인오술(火) → 겁살 해 → 도화 묘, 역마 신, 장성 오, 반안 미, 화개 술
//! - 사유축(金) → 겁살 인 → 도화 오, 역마 해, 장성 유, 반안 술, 화개 축
//! - 해묘미(木) → 겁살 신 → 도화 자, 역마 사, 장성 묘, 반안 진, 화개 미
//!
//! 해당 신살의 지지가 사주 원국 어느 기둥(연/월/일/시)에 등장하면 발현.
//! `positions`에 기둥 위치를 모아주고, `intensity = positions.len()` 으로
//! 단순화 (잠재 0 → 약 1 → 중 2 → 강 3+).
//!
//! **백호살**은 60갑자 중 다음 7개 갑자가 어느 기둥(연/월/일/시)에든
//! 등장하면 발현: 갑진·을미·병술·정축·무진·임술·계축.
//!
//! **천을귀인**은 일간(`day.stem`) 기준 두 지지가 어느 기둥에든 등장하면 발현.

use crate::types::{Branch, FourPillars, Pillar, Stem};
use serde::Serialize;

/// 신살 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum ShinsalKind {
    /// 도화살(桃花殺) = 년살(年殺). 매력 자본·스타성.
    Dohwa,
    /// 역마살(驛馬殺). 글로벌·이동·확장.
    Yeokma,
    /// 장성살(將星殺). 리더십·결단력.
    Jangseong,
    /// 반안살(攀鞍殺). 명예·학위·빠른 출세.
    Banan,
    /// 화개살(華蓋殺). 학문·예술·종교적 깊이.
    Hwagae,
    /// 백호살(白虎殺). 장애를 돌파하는 폭발적 카리스마.
    Baekho,
    /// 천을귀인(天乙貴人). 위기를 회복으로 바꾸는 조력자 에너지.
    Cheoneul,
}

impl ShinsalKind {
    pub fn korean(self) -> &'static str {
        match self {
            ShinsalKind::Dohwa => "도화살",
            ShinsalKind::Yeokma => "역마살",
            ShinsalKind::Jangseong => "장성살",
            ShinsalKind::Banan => "반안살",
            ShinsalKind::Hwagae => "화개살",
            ShinsalKind::Baekho => "백호살",
            ShinsalKind::Cheoneul => "천을귀인",
        }
    }

    /// 보고서 §6의 "현대적 재해석" 통변 카피.
    pub fn modern_take(self) -> &'static str {
        match self {
            ShinsalKind::Dohwa => {
                "대중을 사로잡는 매력 자본과 스타성. 연예·인플루언서·마케팅·정치에서 \
                 대체하기 어려운 무기로 작용하나, 사주 구성이 불리할 때는 이성 관계 \
                 구설로 흐를 수 있어 자기 표현의 결을 정돈하는 자세가 필요합니다."
            }
            ShinsalKind::Yeokma => {
                "고향과 익숙함을 떠나 넓게 움직이는 활력. 외교·다국적·무역·IT 통신·\
                 세일즈처럼 활동 반경이 넓은 영역에서 강한 성과로 환원되는 길살입니다."
            }
            ShinsalKind::Jangseong => {
                "무리의 중심에 서서 책임을 지는 자리. 결단력 있는 리더십이 자연스럽고, \
                 주변과의 마찰은 피하기 어렵지만 흐트러진 판을 정리하는 데 강합니다."
            }
            ShinsalKind::Banan => {
                "명예·학위·자격증을 비교적 일찍 손에 쥐는 출세 기운. 빠른 안착과 \
                 정리정돈의 능력이 강점이지만, 지위가 올라갈수록 허세를 경계해야 \
                 추락을 피합니다."
            }
            ShinsalKind::Hwagae => {
                "내면의 고독과 깊이를 동력으로 학문·예술·종교·콘텐츠 영역에서 \
                 큰 잠재력을 발휘합니다. 화려한 무대 뒤의 집중된 시간이 결과의 무게를 \
                 만들어주는 자리입니다."
            }
            ShinsalKind::Baekho => {
                "장애를 정면으로 돌파하는 폭발적 에너지와 카리스마. 외과의·검사·운동\
                 선수·자수성가형 사업가에게 발현되는 강한 추진력이며, 큰 책임을 \
                 감당할 때 가장 빛납니다."
            }
            ShinsalKind::Cheoneul => {
                "가장 어두운 순간에 결정적 도움이 도래하는 사주 내 안전장치. 큰 위기를 \
                 회복의 변곡점으로 바꾸는 조력자 에너지로, 흉운에서도 바닥을 치지 \
                 않게 받쳐줍니다."
            }
        }
    }

    /// 흉살 계열인지(전통적 분류). UI에서 톤 차별화에 사용 가능.
    /// (천을귀인은 길살, 도화/역마/장성/반안/화개/백호는 전통적으로 흉/중간 — 그러나 본 엔진의 통변은 모두 현대적 긍정 톤.)
    pub fn is_traditionally_inauspicious(self) -> bool {
        !matches!(self, ShinsalKind::Cheoneul)
    }
}

/// 신살이 자리한 사주 기둥.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ShinsalPosition {
    Year,
    Month,
    Day,
    Hour,
}

impl ShinsalPosition {
    pub fn korean(self) -> &'static str {
        match self {
            ShinsalPosition::Year => "연주",
            ShinsalPosition::Month => "월주",
            ShinsalPosition::Day => "일주",
            ShinsalPosition::Hour => "시주",
        }
    }
}

/// 발현된 신살 1종에 대한 정보.
#[derive(Debug, Clone, Serialize)]
pub struct Shinsal {
    pub kind: ShinsalKind,
    /// 사주 원국에서 신살이 자리한 기둥들 (1개 이상).
    pub positions: Vec<ShinsalPosition>,
    /// 단순화된 강도 = positions.len(). 1=잠재, 2=중, 3+=강.
    pub intensity: u8,
    /// 보고서 §6 톤의 현대적 통변 (UI에서 그대로 노출 가능).
    pub modern_take: &'static str,
}

/// 사주 원국에서 7종 신살 발현 여부를 판정.
///
/// `has_birth_time = false` 이면 시주를 검사 대상에서 제외.
/// 결과 Vec은 발현된 신살만 포함 (intensity ≥ 1). 순서는 enum 정의 순.
pub fn analyze(pillars: &FourPillars, has_birth_time: bool) -> Vec<Shinsal> {
    let mut out = Vec::new();

    // ── 12신살 5종 (년지 기준) ──────────────────────
    // 12신살 전체 매핑 (offset from 겁살 인덱스, 0..11): 겁살 0 / 재살 1 /
    // 천살 2 / 지살 3 / 도화(년살) 4 / 월살 5 / 망신 6 / 장성 7 / 반안 8 /
    // 역마 9 / 육해 10 / 화개 11. v0.2.0에서는 4·7·8·9·11만 활성.
    let year_branch = pillars.year.branch;
    let kk = kkupsal_index_for_year(year_branch);

    for (kind, idx_offset) in [
        (ShinsalKind::Dohwa, 4),
        (ShinsalKind::Jangseong, 7),
        (ShinsalKind::Banan, 8),
        (ShinsalKind::Yeokma, 9),
        (ShinsalKind::Hwagae, 11),
    ] {
        let target = Branch::ALL[(kk + idx_offset) % 12];
        let positions = scan_branch_positions(pillars, target, has_birth_time);
        if !positions.is_empty() {
            out.push(Shinsal {
                kind,
                intensity: positions.len() as u8,
                positions,
                modern_take: kind.modern_take(),
            });
        }
    }

    // ── 백호살 ──────────────────────
    let baekho_positions = scan_baekho_positions(pillars, has_birth_time);
    if !baekho_positions.is_empty() {
        out.push(Shinsal {
            kind: ShinsalKind::Baekho,
            intensity: baekho_positions.len() as u8,
            positions: baekho_positions,
            modern_take: ShinsalKind::Baekho.modern_take(),
        });
    }

    // ── 천을귀인 ──────────────────────
    let cheoneul_branches = cheoneul_branches_for(pillars.day.stem);
    let mut cheoneul_positions = Vec::new();
    for &b in cheoneul_branches.iter() {
        cheoneul_positions.extend(scan_branch_positions(pillars, b, has_birth_time));
    }
    cheoneul_positions.sort_by_key(|p| *p as u8);
    cheoneul_positions.dedup();
    if !cheoneul_positions.is_empty() {
        out.push(Shinsal {
            kind: ShinsalKind::Cheoneul,
            intensity: cheoneul_positions.len() as u8,
            positions: cheoneul_positions,
            modern_take: ShinsalKind::Cheoneul.modern_take(),
        });
    }

    out
}

/// 년지가 속한 삼합의 겁살(劫殺) 위치 = 12신살 인덱스 0의 자리.
/// 삼합 첫 글자 idx에서 -3한 위치(= 정충관계).
fn kkupsal_index_for_year(year_branch: Branch) -> usize {
    use Branch::*;
    let trine_first = match year_branch {
        // 신자진 (수국) — 첫 글자 신
        Sin | Ja | Jin => Sin,
        // 인오술 (화국) — 첫 글자 인
        In | O | Sul => In,
        // 사유축 (금국) — 첫 글자 사
        Sa | Yu | Chuk => Sa,
        // 해묘미 (목국) — 첫 글자 해
        Hae | Myo | Mi => Hae,
    };
    let i = trine_first.index() as i32;
    ((i - 3).rem_euclid(12)) as usize
}

/// 사주 4(또는 3) 기둥 중 어느 기둥의 _지지_가 target과 일치하는지 스캔.
fn scan_branch_positions(
    pillars: &FourPillars,
    target: Branch,
    has_birth_time: bool,
) -> Vec<ShinsalPosition> {
    let mut out = Vec::new();
    if pillars.year.branch == target {
        out.push(ShinsalPosition::Year);
    }
    if pillars.month.branch == target {
        out.push(ShinsalPosition::Month);
    }
    if pillars.day.branch == target {
        out.push(ShinsalPosition::Day);
    }
    if has_birth_time && pillars.hour.branch == target {
        out.push(ShinsalPosition::Hour);
    }
    out
}

/// 백호 7갑자.
const BAEKHO_PILLARS: [(Stem, Branch); 7] = [
    (Stem::Gap, Branch::Jin),    // 갑진
    (Stem::Eul, Branch::Mi),     // 을미
    (Stem::Byeong, Branch::Sul), // 병술
    (Stem::Jeong, Branch::Chuk), // 정축
    (Stem::Mu, Branch::Jin),     // 무진
    (Stem::Im, Branch::Sul),     // 임술
    (Stem::Gye, Branch::Chuk),   // 계축
];

fn is_baekho(pillar: Pillar) -> bool {
    BAEKHO_PILLARS
        .iter()
        .any(|(s, b)| *s == pillar.stem && *b == pillar.branch)
}

fn scan_baekho_positions(pillars: &FourPillars, has_birth_time: bool) -> Vec<ShinsalPosition> {
    let mut out = Vec::new();
    if is_baekho(pillars.year) {
        out.push(ShinsalPosition::Year);
    }
    if is_baekho(pillars.month) {
        out.push(ShinsalPosition::Month);
    }
    if is_baekho(pillars.day) {
        out.push(ShinsalPosition::Day);
    }
    if has_birth_time && is_baekho(pillars.hour) {
        out.push(ShinsalPosition::Hour);
    }
    out
}

/// 일간별 천을귀인 두 지지.
fn cheoneul_branches_for(day_stem: Stem) -> [Branch; 2] {
    use Branch::*;
    match day_stem {
        Stem::Gap | Stem::Mu | Stem::Gyeong => [Chuk, Mi],
        Stem::Eul | Stem::Gi => [Ja, Sin],
        Stem::Byeong | Stem::Jeong => [Hae, Yu],
        Stem::Im | Stem::Gye => [Sa, Myo],
        Stem::Sin => [In, O],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Pillar;

    fn pillars(year: (Stem, Branch), month: (Stem, Branch), day: (Stem, Branch), hour: (Stem, Branch)) -> FourPillars {
        FourPillars {
            year: Pillar::new(year.0, year.1),
            month: Pillar::new(month.0, month.1),
            day: Pillar::new(day.0, day.1),
            hour: Pillar::new(hour.0, hour.1),
        }
    }

    // ── 12신살 anchor ──────────────────────────────────────────

    #[test]
    fn dohwa_for_jaja_year_is_yu() {
        // 년지 자(자) → 신자진 → 도화 유
        let p = pillars(
            (Stem::Gap, Branch::Ja),
            (Stem::Eul, Branch::Yu), // 월지에 유
            (Stem::Byeong, Branch::In),
            (Stem::Jeong, Branch::Myo),
        );
        let r = analyze(&p, true);
        let dohwa = r.iter().find(|s| s.kind == ShinsalKind::Dohwa).expect("도화 발현");
        assert_eq!(dohwa.positions, vec![ShinsalPosition::Month]);
        assert_eq!(dohwa.intensity, 1);
    }

    #[test]
    fn yeokma_for_inin_year_is_sin() {
        // 년지 인 → 인오술 → 역마 신
        let p = pillars(
            (Stem::Gap, Branch::In),
            (Stem::Im, Branch::Sin), // 월지에 신
            (Stem::Byeong, Branch::Ja),
            (Stem::Jeong, Branch::Myo),
        );
        let r = analyze(&p, true);
        let yeokma = r.iter().find(|s| s.kind == ShinsalKind::Yeokma).expect("역마 발현");
        assert_eq!(yeokma.positions, vec![ShinsalPosition::Month]);
    }

    #[test]
    fn jangseong_for_inin_year_is_o() {
        // 년지 인 → 인오술 → 장성 오
        let p = pillars(
            (Stem::Gap, Branch::In),
            (Stem::Eul, Branch::Yu),
            (Stem::Byeong, Branch::O), // 일지에 오
            (Stem::Jeong, Branch::Myo),
        );
        let r = analyze(&p, true);
        let js = r.iter().find(|s| s.kind == ShinsalKind::Jangseong).expect("장성 발현");
        assert_eq!(js.positions, vec![ShinsalPosition::Day]);
    }

    #[test]
    fn banan_for_inin_year_is_mi() {
        // 년지 인 → 인오술 → 반안 미
        let p = pillars(
            (Stem::Gap, Branch::In),
            (Stem::Eul, Branch::Yu),
            (Stem::Byeong, Branch::Ja),
            (Stem::Jeong, Branch::Mi), // 시지에 미
        );
        let r = analyze(&p, true);
        let bn = r.iter().find(|s| s.kind == ShinsalKind::Banan).expect("반안 발현");
        assert_eq!(bn.positions, vec![ShinsalPosition::Hour]);
    }

    #[test]
    fn hwagae_for_inin_year_is_sul() {
        // 년지 인 → 인오술 → 화개 술
        let p = pillars(
            (Stem::Gap, Branch::In),
            (Stem::Eul, Branch::Yu),
            (Stem::Byeong, Branch::Sul), // 일지에 술
            (Stem::Jeong, Branch::Myo),
        );
        let r = analyze(&p, true);
        let hg = r.iter().find(|s| s.kind == ShinsalKind::Hwagae).expect("화개 발현");
        assert_eq!(hg.positions, vec![ShinsalPosition::Day]);
    }

    /// 4 삼합 모두에서 도화가 정상 매핑되는지.
    #[test]
    fn dohwa_anchor_per_trine() {
        // 신자진 → 도화 유
        // 인오술 → 도화 묘
        // 사유축 → 도화 오
        // 해묘미 → 도화 자
        let cases = [
            (Branch::Ja, Branch::Yu),
            (Branch::O, Branch::Myo),
            (Branch::Yu, Branch::O),
            (Branch::Myo, Branch::Ja),
        ];
        for (year_branch, expected_dohwa) in cases {
            let p = pillars(
                (Stem::Gap, year_branch),
                (Stem::Eul, expected_dohwa),
                (Stem::Byeong, Branch::In),
                (Stem::Jeong, Branch::In),
            );
            let r = analyze(&p, true);
            let found = r.iter().any(|s| s.kind == ShinsalKind::Dohwa);
            assert!(
                found,
                "year_branch={:?} expected dohwa branch={:?}",
                year_branch, expected_dohwa
            );
        }
    }

    /// has_birth_time = false 이면 시지의 도화는 검출되지 않는다.
    #[test]
    fn skip_hour_when_no_birth_time() {
        let p = pillars(
            (Stem::Gap, Branch::Ja),
            (Stem::Eul, Branch::In),
            (Stem::Byeong, Branch::In),
            (Stem::Jeong, Branch::Yu), // 시지에만 도화 유
        );
        let r = analyze(&p, false);
        assert!(!r.iter().any(|s| s.kind == ShinsalKind::Dohwa));
    }

    // ── 백호 anchor ──────────────────────────────────────────

    #[test]
    fn baekho_gapjin_day() {
        // 갑진 일주 → 백호 발현
        let p = pillars(
            (Stem::Gye, Branch::Hae),
            (Stem::Eul, Branch::Myo),
            (Stem::Gap, Branch::Jin), // 갑진 일주
            (Stem::Byeong, Branch::In),
        );
        let r = analyze(&p, true);
        let bh = r.iter().find(|s| s.kind == ShinsalKind::Baekho).expect("백호 발현");
        assert_eq!(bh.positions, vec![ShinsalPosition::Day]);
    }

    #[test]
    fn baekho_all_seven_are_detected() {
        // 7갑자 각각이 일주에 있을 때 모두 검출.
        let cases = BAEKHO_PILLARS;
        for (s, b) in cases {
            let p = pillars(
                (Stem::Gye, Branch::Hae),
                (Stem::Eul, Branch::Myo),
                (s, b),
                (Stem::Byeong, Branch::In),
            );
            let r = analyze(&p, true);
            assert!(
                r.iter().any(|x| x.kind == ShinsalKind::Baekho),
                "{:?}{:?} 일주에 백호 검출 실패",
                s,
                b
            );
        }
    }

    #[test]
    fn baekho_in_year_pillar() {
        // 백호는 어느 기둥이든 발현 — 연주에 갑진.
        let p = pillars(
            (Stem::Gap, Branch::Jin), // 갑진 연주
            (Stem::Eul, Branch::Myo),
            (Stem::Im, Branch::Sin),
            (Stem::Byeong, Branch::In),
        );
        let r = analyze(&p, true);
        let bh = r.iter().find(|s| s.kind == ShinsalKind::Baekho).expect("백호 발현");
        assert_eq!(bh.positions, vec![ShinsalPosition::Year]);
    }

    /// 백호가 아닌 일반 갑자(예: 갑자)는 검출되지 않는다.
    #[test]
    fn non_baekho_pillar_is_skipped() {
        let p = pillars(
            (Stem::Gap, Branch::Ja),
            (Stem::Eul, Branch::Chuk),
            (Stem::Byeong, Branch::In),
            (Stem::Jeong, Branch::Myo),
        );
        let r = analyze(&p, true);
        assert!(!r.iter().any(|s| s.kind == ShinsalKind::Baekho));
    }

    // ── 천을귀인 anchor ──────────────────────────────────────────

    #[test]
    fn cheoneul_for_gap_day_master_is_chuk_or_mi() {
        // 갑일간 → 천을귀인 축·미
        let p = pillars(
            (Stem::Im, Branch::In),
            (Stem::Gye, Branch::Chuk), // 월지 축
            (Stem::Gap, Branch::Ja),
            (Stem::Eul, Branch::Mi), // 시지 미
        );
        let r = analyze(&p, true);
        let ce = r.iter().find(|s| s.kind == ShinsalKind::Cheoneul).expect("귀인 발현");
        // 두 기둥 모두 잡힘.
        assert_eq!(ce.positions.len(), 2);
        assert!(ce.positions.contains(&ShinsalPosition::Month));
        assert!(ce.positions.contains(&ShinsalPosition::Hour));
    }

    /// 천을귀인 5 일간 그룹 anchor 잠금.
    #[test]
    fn cheoneul_branch_table() {
        assert_eq!(cheoneul_branches_for(Stem::Gap), [Branch::Chuk, Branch::Mi]);
        assert_eq!(cheoneul_branches_for(Stem::Eul), [Branch::Ja, Branch::Sin]);
        assert_eq!(cheoneul_branches_for(Stem::Byeong), [Branch::Hae, Branch::Yu]);
        assert_eq!(cheoneul_branches_for(Stem::Im), [Branch::Sa, Branch::Myo]);
        assert_eq!(cheoneul_branches_for(Stem::Sin), [Branch::In, Branch::O]);
    }

    // ── 통합 ──────────────────────────────────────────

    /// 신살 0개 케이스 — 모두 부재.
    #[test]
    fn no_shinsal_present() {
        // 일부러 모든 신살을 피한 사주: 년지 인(인오술 → 도화묘/장성오/반안미/역마신/화개술),
        // 다른 기둥은 자축... 모두 인오술의 12신살에 안 닿게.
        let p = pillars(
            (Stem::Im, Branch::In),
            (Stem::Gye, Branch::Hae),
            (Stem::Gap, Branch::Ja),
            (Stem::Eul, Branch::Chuk),
        );
        // 갑자 일주는 백호 아님. 갑일간 천을귀인은 축/미 — 시지 축이 잡힘.
        let r = analyze(&p, true);
        // 천을귀인은 잡힘
        assert!(r.iter().any(|s| s.kind == ShinsalKind::Cheoneul));
        // 12신살 5종은 모두 부재.
        for kind in [
            ShinsalKind::Dohwa,
            ShinsalKind::Yeokma,
            ShinsalKind::Jangseong,
            ShinsalKind::Banan,
            ShinsalKind::Hwagae,
        ] {
            assert!(!r.iter().any(|s| s.kind == kind), "{:?} should be absent", kind);
        }
        assert!(!r.iter().any(|s| s.kind == ShinsalKind::Baekho));
    }

    /// modern_take 카피가 모든 종류에 대해 비어있지 않다.
    #[test]
    fn all_modern_take_copy_present() {
        for k in [
            ShinsalKind::Dohwa,
            ShinsalKind::Yeokma,
            ShinsalKind::Jangseong,
            ShinsalKind::Banan,
            ShinsalKind::Hwagae,
            ShinsalKind::Baekho,
            ShinsalKind::Cheoneul,
        ] {
            assert!(!k.modern_take().is_empty(), "{:?} modern_take empty", k);
            assert!(!k.korean().is_empty(), "{:?} korean empty", k);
        }
    }
}
