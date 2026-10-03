cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use chrono::{Datelike, Days, Utc};
        use indexmap::{IndexMap, IndexSet};
        use kid_types::{TaskDetails, TaskInfos};
        use kid_types::TaskAuthors;
        use kid_types::TaskAvailability;
        use kid_types::TaskCategory;
        use kid_types::TaskTimeEstimate;
        use std::collections::BTreeMap;
    }
}

use chrono::NaiveDate;

use kid_types::Uuid;
use kid_types::task;

use crate::{DeadlineGroup, Urgency};

pub type UpcomingGroups = Vec<(DeadlineGroup, Vec<(Uuid, task::Infos, Urgency, NaiveDate)>)>;
pub type UpcomingBacklog = Vec<(Uuid, task::Infos)>;

#[cfg(feature = "ssr")]
pub type QuickWinsGroups = Vec<(TaskTimeEstimate, Vec<(Uuid, task::Infos)>)>;

/// Assign a deadline group based on the task's attention date.
///
/// Overdue is always based on the actual `due_date`. For non-overdue
/// tasks, the attention date (`due_date` minus `lead_days` eligible
/// days for the given `availability`) determines the bucket.
#[cfg(feature = "ssr")]
pub(super) fn deadline_group(
    due_date: NaiveDate,
    estimate: Option<TaskTimeEstimate>,
    availability: TaskAvailability,
    today: NaiveDate,
    tomorrow: NaiveDate,
    this_sunday: NaiveDate,
    next_sunday: NaiveDate,
) -> DeadlineGroup {
    if due_date < today {
        return DeadlineGroup::Overdue;
    }

    // Attention date: due - lead eligible days.
    let group_date = match estimate {
        Some(est) if est.lead_days() > 0 => {
            let lead = est.lead_days();
            let mut remaining = lead;
            let mut date = due_date;
            while remaining > 0 {
                date = date.pred_opt().expect("date underflow");
                if availability.is_eligible(date) {
                    remaining -= 1;
                }
            }
            date
        }
        _ => due_date,
    };

    if group_date <= today {
        DeadlineGroup::Today
    } else if group_date <= tomorrow {
        DeadlineGroup::Tomorrow
    } else if group_date <= this_sunday {
        DeadlineGroup::ThisWeek
    } else if group_date <= next_sunday {
        DeadlineGroup::NextWeek
    } else {
        DeadlineGroup::Later
    }
}

/// Group open tasks by temporal urgency for the Upcoming view.
///
/// Returns `(groups, backlog)` where `backlog` contains open tasks
/// without any date — unaffected by context filters (UXDR).
///
/// Each grouped task carries an [`Urgency`] level that controls the
/// checkbox border accent (UXDR: upcoming-urgency-checkbox).
#[cfg(feature = "ssr")]
pub fn group_upcoming<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
    today: NaiveDate,
) -> (UpcomingGroups, UpcomingBacklog) {
    let tomorrow = today + Days::new(1);
    // ISO 8601 week boundaries (Monday = start of week).
    let days_until_sunday = 6 - today.weekday().num_days_from_monday();
    let this_sunday = today + Days::new(days_until_sunday as u64);
    let next_sunday = this_sunday + Days::new(7);

    let mut backlog: UpcomingBacklog = Vec::new();
    // (id, info, group, sort_date, urgency, soft)
    //
    // urgency: checkbox accent per UXDR upcoming-urgency-checkbox.
    // soft: true when the task landed in an earlier group because of
    //   start_date rather than attention_date — these sort after the
    //   "hard" attention-driven tasks within the same group.
    let mut items: Vec<(Uuid, task::Infos, DeadlineGroup, NaiveDate, Urgency, bool)> = Vec::new();

    for (id, task) in tasks {
        if task.info().is_done() {
            continue;
        }
        let due = task.due_date().map(|d| d.date.date_naive());
        let start = task.start_date().map(|d| d.date.date_naive());

        // Inclusion: open AND (has due_date OR start_date <= today).
        // Everything else is backlog.
        if due.is_none() && start.is_none_or(|s| s > today) {
            backlog.push((id.to_owned(), task.info().to_owned()));
            continue;
        }

        // Group by attention date; sort within groups by actual
        // due_date (UXDR).
        let (group, sort_date, urgency, soft) = if let Some(due_date) = due {
            let est = task.time_estimate().copied();
            let avail = *task.availability();
            let attention_group = deadline_group(
                due_date,
                est,
                avail,
                today,
                tomorrow,
                this_sunday,
                next_sunday,
            );
            // Compute the raw attention date (without start_date
            // override) for soft-shift comparison.
            let attention_date = match est {
                Some(e) if e.lead_days() > 0 => {
                    let lead = e.lead_days();
                    let mut remaining = lead;
                    let mut d = due_date;
                    while remaining > 0 {
                        d = d.pred_opt().expect("date underflow");
                        if avail.is_eligible(d) {
                            remaining -= 1;
                        }
                    }
                    Some(d)
                }
                _ => None,
            };
            let effective_attention = attention_date.unwrap_or(due_date);

            // If start_date < attention_date, the user wants to begin
            // earlier than the computed lead time demands. Use
            // start_date for group assignment but mark as "soft" so
            // these sort after attention-driven tasks in the same group.
            //
            // The effective start day is the first availability-eligible
            // day from today (or start_date, whichever is later). This
            // prevents e.g. a WeekendOnly task from landing in "Today"
            // on a Wednesday just because its start_date has passed.
            let (group, soft) = match start {
                Some(sd)
                    if sd < effective_attention
                        && !matches!(attention_group, DeadlineGroup::Overdue) =>
                {
                    let mut eff = sd.max(today);
                    while !avail.is_eligible(eff) {
                        eff = eff.succ_opt().expect("date overflow");
                    }
                    let start_group = if eff <= today {
                        DeadlineGroup::Today
                    } else if eff <= tomorrow {
                        DeadlineGroup::Tomorrow
                    } else if eff <= this_sunday {
                        DeadlineGroup::ThisWeek
                    } else if eff <= next_sunday {
                        DeadlineGroup::NextWeek
                    } else {
                        DeadlineGroup::Later
                    };
                    // Only shift if start_date actually pulls into an
                    // earlier group; otherwise keep attention_group.
                    if (start_group as u8) < (attention_group as u8) {
                        (start_group, true)
                    } else {
                        (attention_group, false)
                    }
                }
                _ => (attention_group, false),
            };

            // Urgency: checkbox accent per UXDR upcoming-urgency-checkbox.
            // Later is too far out for urgency signals — always None.
            use kid_types::TaskTimeEstimate as TTE;
            let urgency = if matches!(group, DeadlineGroup::Later) {
                Urgency::None
            } else if soft {
                match est {
                    Some(TTE::Min15 | TTE::Min30 | TTE::Min45 | TTE::Hours1 | TTE::Hours2) => {
                        Urgency::None
                    }
                    _ => Urgency::Standard, // HalfDay, Day1, Day2, or None
                }
            } else {
                match est {
                    Some(TTE::Min15 | TTE::Min30 | TTE::Min45 | TTE::Hours1 | TTE::Hours2) => {
                        Urgency::Hard
                    }
                    _ => Urgency::High, // HalfDay, Day1, Day2, or None
                }
            };

            (group, due_date, urgency, soft)
        } else {
            // start_date <= today, no due_date → Ready to Start
            (
                DeadlineGroup::ReadyToStart,
                start.unwrap(),
                Urgency::None,
                false,
            )
        };

        items.push((
            id.to_owned(),
            task.info().to_owned(),
            group,
            sort_date,
            urgency,
            soft,
        ));
    }

    // Sort: group order, then within-group rules per UXDR.
    // Within each group, "hard" tasks (attention-driven) come before
    // "soft" tasks (start_date-driven).
    items.sort_by(|a, b| {
        (a.2 as u8)
            .cmp(&(b.2 as u8))
            .then_with(|| a.5.cmp(&b.5)) // false < true → hard before soft
            .then_with(|| {
                let pri = |info: &task::Infos| -> u8 {
                    info.priority().map(|p| *p as u8).unwrap_or(u8::MAX)
                };
                match a.2 {
                    // Today/Tomorrow: single-day groups → priority descending, then UUID.
                    DeadlineGroup::Today | DeadlineGroup::Tomorrow => {
                        pri(&a.1).cmp(&pri(&b.1)).then_with(|| a.0.cmp(&b.0))
                    }
                    // All others: date ascending, then priority, then UUID.
                    _ => {
                        a.3.cmp(&b.3)
                            .then_with(|| pri(&a.1).cmp(&pri(&b.1)))
                            .then_with(|| a.0.cmp(&b.0))
                    }
                }
            })
    });

    // Chunk into contiguous groups (items are already in group order).
    let mut groups: UpcomingGroups = Vec::new();
    for (id, info, group, sort_date, urgency, _) in items {
        if groups.last().is_none_or(|(key, _)| *key != group) {
            groups.push((group, Vec::new()));
        }
        groups
            .last_mut()
            .unwrap()
            .1
            .push((id, info, urgency, sort_date));
    }

    // Sort backlog: priority descending (A first), then UUID (creation order).
    backlog.sort_by(|a, b| {
        let pri =
            |info: &task::Infos| -> u8 { info.priority().map(|p| *p as u8).unwrap_or(u8::MAX) };
        pri(&a.1).cmp(&pri(&b.1)).then_with(|| a.0.cmp(&b.0))
    });

    (groups, backlog)
}

#[cfg(feature = "ssr")]
pub(super) fn group_by_category(
    list: Vec<(Uuid, task::Infos)>,
) -> IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>> {
    let mut btree: BTreeMap<TaskCategory, Vec<(Uuid, task::Infos)>> = BTreeMap::new();
    for item in list {
        btree
            .entry(item.1.category().parse().unwrap())
            .or_default()
            .push(item);
    }
    btree.into_iter().collect()
}

/// Group open tasks by time estimate, sorted by priority within each group.
///
/// Returns groups in ascending estimate order (Min15 → Day2).
/// Within each group: priority descending (A before B before C),
/// then UUID ascending (creation order).
#[cfg(feature = "ssr")]
pub fn group_quick_wins<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
) -> Vec<(TaskTimeEstimate, Vec<(Uuid, task::Infos)>)> {
    let list: Vec<_> = tasks
        .filter(|(_, task)| !task.info().is_done() && task.time_estimate().is_some())
        .map(|(id, task)| {
            (
                id.to_owned(),
                task.info().to_owned(),
                task.time_estimate().cloned().unwrap(),
            )
        })
        .collect();
    let mut btree: BTreeMap<TaskTimeEstimate, Vec<(Uuid, task::Infos)>> = BTreeMap::new();
    for (id, info, te) in list {
        btree.entry(te).or_default().push((id, info));
    }
    let mut groups: Vec<_> = btree.into_iter().collect();
    for (_, tasks) in &mut groups {
        tasks.sort_by(|(id_a, a), (id_b, b)| {
            let pri =
                |info: &task::Infos| -> u8 { info.priority().map(|p| *p as u8).unwrap_or(u8::MAX) };
            pri(a).cmp(&pri(b)).then_with(|| id_a.cmp(id_b))
        });
    }
    groups
}

/// Group completed tasks by category, sorted by completion date within each.
///
/// Categories in alphabetical order (via BTreeMap). Within each
/// category: most recently completed first (completion date descending).
#[cfg(feature = "ssr")]
pub(super) fn group_finished<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
) -> IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>> {
    let list: Vec<_> = tasks
        .filter(|(_, task)| task.info().is_done())
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    let mut groups = group_by_category(list);
    for tasks in groups.values_mut() {
        tasks.sort_by(|(_, a), (_, b)| b.since().cmp(a.since()));
    }
    groups
}

/// Group open tasks by category, sorted by UUID (creation order) within each.
///
/// Categories in alphabetical order (via BTreeMap). Within each
/// category: UUID ascending (oldest first — UUID v7 encodes creation time).
#[cfg(feature = "ssr")]
pub(super) fn group_all_open<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
) -> IndexMap<TaskCategory, Vec<(Uuid, task::Infos)>> {
    let mut list: Vec<_> = tasks
        .filter(|(_, task)| !task.info().is_done())
        .map(|(id, task)| (id.to_owned(), task.info().to_owned()))
        .collect();
    list.sort_by_key(|(id, _)| *id);
    group_by_category(list)
}

/// Collect recently changed tasks sorted by last-change descending.
///
/// Includes all tasks within the 3-day calendar window
/// (`today - 2` through `today`). Beyond that, collects up to
/// `extra_days` distinct older days that actually contain data
/// (empty days are skipped).
#[cfg(feature = "ssr")]
pub(super) fn group_recently_changed<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
    today: NaiveDate,
    extra_days: u32,
) -> Vec<super::RecentChange> {
    let calendar_cutoff = today.checked_sub_days(Days::new(2)).unwrap();
    let mut all: Vec<_> = tasks
        .map(|(id, task)| {
            let last_change = task
                .authors()
                .values()
                .flatten()
                .max()
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|| task.info().since().with_timezone(&Utc));
            let day = last_change.date_naive();
            let authors = TaskAuthors::from(task.authors());
            let ai_involved = authors.iter().any(|(a, _)| a.starts_with("ai:"));
            let ai_last = authors
                .iter()
                .max_by_key(|(_, ts)| ts)
                .is_some_and(|(a, _)| a.starts_with("ai:"));
            (
                last_change,
                day,
                super::RecentChange {
                    id: id.to_owned(),
                    info: task.info().to_owned(),
                    authors,
                    last_changed: last_change.fixed_offset(),
                    ai_last,
                    ai_involved,
                },
            )
        })
        .collect();
    all.sort_by(|(a, _, _), (b, _, _)| b.cmp(a));
    let mut older_days: IndexSet<NaiveDate> = IndexSet::new();
    all.into_iter()
        .filter(|(_, day, _)| {
            if *day >= calendar_cutoff {
                true
            } else if older_days.len() < extra_days as usize {
                older_days.insert(*day);
                true
            } else {
                older_days.contains(day)
            }
        })
        .map(|(_, _, rc)| rc)
        .collect()
}

/// Collect tasks changed within the last `days` days (inclusive of
/// `today`), sorted by last-change descending. Unlike
/// `group_recently_changed`, this uses a hard date cutoff — no calendar
/// window, no extra-days padding for empty days.
#[cfg(feature = "ssr")]
pub fn group_recently_changed_since<'a>(
    tasks: impl Iterator<Item = (&'a Uuid, &'a kid_types::Task)>,
    today: NaiveDate,
    days: u32,
) -> Vec<super::RecentChange> {
    let cutoff = today.checked_sub_days(Days::new(days as u64)).unwrap();
    let mut all: Vec<_> = tasks
        .filter_map(|(id, task)| {
            let last_change = task
                .authors()
                .values()
                .flatten()
                .max()
                .map(|t| t.with_timezone(&Utc))
                .unwrap_or_else(|| task.info().since().with_timezone(&Utc));
            if last_change.date_naive() < cutoff {
                return None;
            }
            let authors = TaskAuthors::from(task.authors());
            let ai_involved = authors.iter().any(|(a, _)| a.starts_with("ai:"));
            let ai_last = authors
                .iter()
                .max_by_key(|(_, ts)| ts)
                .is_some_and(|(a, _)| a.starts_with("ai:"));
            Some(super::RecentChange {
                id: id.to_owned(),
                info: task.info().to_owned(),
                authors,
                last_changed: last_change.fixed_offset(),
                ai_last,
                ai_involved,
            })
        })
        .collect();
    all.sort_by_key(|change| std::cmp::Reverse(change.last_changed));
    all
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use kid_types::TaskAvailability as Avail;
    use kid_types::TaskTimeEstimate as Est;

    /// Week anchors for a given `today`.
    fn week_bounds(today: NaiveDate) -> (NaiveDate, NaiveDate) {
        use chrono::Datelike;
        let days_until_sunday = 6 - today.weekday().num_days_from_monday();
        let this_sunday = today + chrono::Days::new(days_until_sunday as u64);
        let next_sunday = this_sunday + chrono::Days::new(7);
        (this_sunday, next_sunday)
    }

    fn group(due: &str, estimate: Option<Est>, availability: Avail, today: &str) -> DeadlineGroup {
        let today = NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap();
        let due = NaiveDate::parse_from_str(due, "%Y-%m-%d").unwrap();
        let tomorrow = today + chrono::Days::new(1);
        let (this_sun, next_sun) = week_bounds(today);
        deadline_group(
            due,
            estimate,
            availability,
            today,
            tomorrow,
            this_sun,
            next_sun,
        )
    }

    // ── Baseline: no estimate, pure due_date grouping ──

    #[test]
    fn overdue_task() {
        // Due yesterday → Overdue
        assert_eq!(
            group("2026-05-08", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Overdue,
        );
    }

    #[test]
    fn due_today_no_estimate() {
        assert_eq!(
            group("2026-05-09", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Today,
        );
    }

    #[test]
    fn due_this_week_no_estimate() {
        // Today Mon May 11 (this_sun=17)
        // Due Wed May 13 → ThisWeek (not today, not tomorrow)
        assert_eq!(
            group("2026-05-13", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn due_next_week_no_estimate() {
        // Today is Sat May 9, next week Mon-Sun = May 11-17
        assert_eq!(
            group("2026-05-14", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn due_later_no_estimate() {
        // Today is Sat May 9, next_sunday = May 17
        // Due May 20 → Later
        assert_eq!(
            group("2026-05-20", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Later,
        );
    }

    // ── Attention date: Anytime ──

    #[test]
    fn day2_anytime_surfaces_earlier() {
        // Today Sat May 9 (this_sun=10, next_sun=17)
        // Due Tue May 19 → without estimate: Later (19 > 17)
        assert_eq!(
            group("2026-05-19", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Later,
        );
        // With Day2: attention = May 17 (Sun) → Next Week (17 <= 17)
        assert_eq!(
            group("2026-05-19", Some(Est::Day2), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn day1_anytime_surfaces_one_day_earlier() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Mon May 18 → without estimate: Next Week (18 <= 24)
        assert_eq!(
            group("2026-05-18", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
        // With Day1: attention = May 17 (Sun) → This Week (17 <= 17)
        assert_eq!(
            group("2026-05-18", Some(Est::Day1), Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn halfday_anytime_no_lead() {
        // HalfDay → lead_days=0 → group_date = due_date → Later
        assert_eq!(
            group(
                "2026-05-20",
                Some(Est::HalfDay),
                Avail::Anytime,
                "2026-05-09"
            ),
            DeadlineGroup::Later,
        );
    }

    // ── Attention date: WeekdayOnly ──

    #[test]
    fn day2_weekday_only_skips_weekend() {
        // Today Wed May 13 (tomorrow=14, this_sun=17, next_sun=24)
        // Due Mon May 18, Day2 WeekdayOnly:
        //   Sun 17 → skip, Sat 16 → skip,
        //   Fri 15 → 1, Thu 14 → 0
        //   attention = May 14 (Thu) = tomorrow → Tomorrow
        assert_eq!(
            group(
                "2026-05-18",
                Some(Est::Day2),
                Avail::WeekdayOnly,
                "2026-05-13"
            ),
            DeadlineGroup::Tomorrow,
        );
    }

    #[test]
    fn day2_weekday_only_due_monday_from_prior_week() {
        // Today Sat May 9, due Mon May 18
        // Day2 WeekdayOnly: attention = Thu May 14 → Next Week
        assert_eq!(
            group(
                "2026-05-18",
                Some(Est::Day2),
                Avail::WeekdayOnly,
                "2026-05-09"
            ),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Attention date: WeekendOnly ──

    #[test]
    fn day2_weekend_only_due_friday_jumps_to_prev_weekend() {
        // Today Sat May 9, due Fri May 22
        // Day2 WeekendOnly: count 2 weekend days back from Fri
        //   Thu→skip … Sun May 17 → 1, Sat May 16 → 0
        //   attention = May 16 → Next Week
        assert_eq!(
            group(
                "2026-05-22",
                Some(Est::Day2),
                Avail::WeekendOnly,
                "2026-05-09"
            ),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn day1_weekend_only_due_friday() {
        // Today Sat May 9, due Fri May 22
        // Day1 WeekendOnly: count 1 weekend day back from Fri
        //   Thu→skip … Sun May 17 → 0
        //   attention = May 17 → Next Week
        assert_eq!(
            group(
                "2026-05-22",
                Some(Est::Day1),
                Avail::WeekendOnly,
                "2026-05-09"
            ),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Attention date reaches today → Today group ──

    #[test]
    fn attention_in_the_past_becomes_today() {
        // Today Wed May 20, due Fri May 22
        // Day2 Anytime: attention = May 20 (today) → Today
        assert_eq!(
            group("2026-05-22", Some(Est::Day2), Avail::Anytime, "2026-05-20"),
            DeadlineGroup::Today,
        );
    }

    #[test]
    fn attention_passed_still_today_not_overdue() {
        // Today Thu May 21, due Fri May 22
        // Day2 Anytime: attention = May 20 (yesterday) → Today
        // (not Overdue — due_date is still in the future)
        assert_eq!(
            group("2026-05-22", Some(Est::Day2), Avail::Anytime, "2026-05-21"),
            DeadlineGroup::Today,
        );
    }

    // ── Overdue ignores attention date ──

    #[test]
    fn overdue_regardless_of_estimate() {
        // Due yesterday, even with Day2 estimate → still Overdue
        assert_eq!(
            group("2026-05-08", Some(Est::Day2), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Overdue,
        );
    }

    #[test]
    fn overdue_with_constrained_availability() {
        assert_eq!(
            group(
                "2026-05-08",
                Some(Est::Day1),
                Avail::WeekendOnly,
                "2026-05-09"
            ),
            DeadlineGroup::Overdue,
        );
    }

    // ── Boundary: group edges ──

    #[test]
    fn due_on_this_sunday_is_this_week() {
        // Today Mon May 11 (this_sun=17)
        // Due Sun May 17 = this_sunday → ThisWeek
        assert_eq!(
            group("2026-05-17", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn due_on_next_sunday_is_next_week() {
        // Today Mon May 11 (next_sun=24)
        // Due Sun May 24 = next_sunday → NextWeek
        assert_eq!(
            group("2026-05-24", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
    }

    #[test]
    fn due_day_after_next_sunday_is_later() {
        // Today Mon May 11 (next_sun=24)
        // Due Mon May 25 → Later
        assert_eq!(
            group("2026-05-25", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::Later,
        );
    }

    // ── Tomorrow ──

    #[test]
    fn due_tomorrow_is_tomorrow() {
        // Today Sat May 9 (this_sun=10)
        // Due Sun May 10 = tomorrow → Tomorrow
        assert_eq!(
            group("2026-05-10", None, Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Tomorrow,
        );
    }

    #[test]
    fn due_tomorrow_midweek() {
        // Today Wed May 13, due Thu May 14 = tomorrow → Tomorrow
        assert_eq!(
            group("2026-05-14", None, Avail::Anytime, "2026-05-13"),
            DeadlineGroup::Tomorrow,
        );
    }

    #[test]
    fn attention_tomorrow_via_estimate() {
        // Today Mon May 12 (this_sun=17)
        // Due Wed May 14, Day2 Anytime:
        //   attention = May 12… no, that's today.
        // Better: Today Tue May 13, due Fri May 16, Day2 Anytime:
        //   attention = May 14 (Wed) = tomorrow → Tomorrow
        assert_eq!(
            group("2026-05-16", Some(Est::Day2), Avail::Anytime, "2026-05-13"),
            DeadlineGroup::Tomorrow,
        );
    }

    #[test]
    fn due_day_after_tomorrow_is_this_week_not_tomorrow() {
        // Today Mon May 11 (this_sun=17)
        // Due Wed May 13 (= today + 2) → ThisWeek, NOT Tomorrow
        assert_eq!(
            group("2026-05-13", None, Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn attention_on_this_sunday_is_this_week() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Tue May 19, Day2: attention = May 17 (Sun) = this_sunday
        // 17 <= 17 → ThisWeek
        assert_eq!(
            group("2026-05-19", Some(Est::Day2), Avail::Anytime, "2026-05-11"),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn attention_on_next_sunday_is_next_week() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Tue May 26, Day2: attention = May 24 (Sun) = next_sunday
        // 24 <= 24 → NextWeek
        assert_eq!(
            group("2026-05-26", Some(Est::Day2), Avail::Anytime, "2026-05-11"),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Today is Sunday: Tomorrow crosses ISO week boundary ──

    #[test]
    fn today_is_sunday_due_tomorrow_is_tomorrow() {
        // Today Sun May 10 (this_sun=10, next_sun=17)
        // Due Mon May 11 = tomorrow → Tomorrow
        // (NOT NextWeek — "what's due tomorrow?" beats ISO weeks)
        assert_eq!(
            group("2026-05-11", None, Avail::Anytime, "2026-05-10"),
            DeadlineGroup::Tomorrow,
        );
    }

    #[test]
    fn today_is_sunday_due_today_is_today() {
        // Today Sun May 10 (this_sun=10)
        assert_eq!(
            group("2026-05-10", None, Avail::Anytime, "2026-05-10"),
            DeadlineGroup::Today,
        );
    }

    // ── Availability has no effect without lead time ──

    #[test]
    fn no_estimate_ignores_availability() {
        // Without estimate, availability changes nothing.
        let base = group("2026-05-20", None, Avail::Anytime, "2026-05-09");
        assert_eq!(
            group("2026-05-20", None, Avail::WeekdayOnly, "2026-05-09"),
            base,
        );
        assert_eq!(
            group("2026-05-20", None, Avail::WeekendOnly, "2026-05-09"),
            base,
        );
    }

    #[test]
    fn sub_day_estimate_ignores_availability() {
        // lead_days=0 → group_date = due_date regardless of availability.
        let base = group(
            "2026-05-20",
            Some(Est::Hours2),
            Avail::Anytime,
            "2026-05-09",
        );
        assert_eq!(
            group(
                "2026-05-20",
                Some(Est::Hours2),
                Avail::WeekdayOnly,
                "2026-05-09"
            ),
            base,
        );
        assert_eq!(
            group(
                "2026-05-20",
                Some(Est::Hours2),
                Avail::WeekendOnly,
                "2026-05-09"
            ),
            base,
        );
    }

    #[test]
    fn all_sub_day_estimates_behave_identically() {
        // Min15..HalfDay all have lead_days=0 → same group as no estimate.
        let today = "2026-05-09";
        let due = "2026-05-20";
        let expected = group(due, None, Avail::Anytime, today);
        for est in [
            Est::Min15,
            Est::Min30,
            Est::Min45,
            Est::Hours1,
            Est::Hours2,
            Est::HalfDay,
        ] {
            assert_eq!(
                group(due, Some(est), Avail::Anytime, today),
                expected,
                "estimate {:?} should match no-estimate grouping",
                est,
            );
        }
    }

    // ── Day1 + WeekdayOnly ──

    #[test]
    fn day1_weekday_only_due_monday_skips_weekend() {
        // Today Wed May 13 (this_sun=17, next_sun=24)
        // Due Mon May 18, Day1 WeekdayOnly:
        //   Sun 17 → skip, Sat 16 → skip, Fri 15 → 0
        //   attention = Fri May 15 → ThisWeek (15 <= 17)
        assert_eq!(
            group(
                "2026-05-18",
                Some(Est::Day1),
                Avail::WeekdayOnly,
                "2026-05-13"
            ),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day1_weekday_only_due_tuesday() {
        // Today Wed May 13 (this_sun=17)
        // Due Tue May 19, Day1 WeekdayOnly:
        //   Mon 18 → eligible → 0
        //   attention = Mon May 18 → NextWeek (18 > 17, 18 <= 24)
        assert_eq!(
            group(
                "2026-05-19",
                Some(Est::Day1),
                Avail::WeekdayOnly,
                "2026-05-13"
            ),
            DeadlineGroup::NextWeek,
        );
    }

    // ── WeekendOnly edge cases ──

    #[test]
    fn day1_weekend_only_due_saturday_jumps_to_prev_sunday() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Sat May 23, Day1 WeekendOnly:
        //   Fri 22→skip, Thu 21→skip, Wed 20→skip,
        //   Tue 19→skip, Mon 18→skip, Sun 17→0
        //   attention = Sun May 17 → ThisWeek (17 <= 17)
        assert_eq!(
            group(
                "2026-05-23",
                Some(Est::Day1),
                Avail::WeekendOnly,
                "2026-05-11"
            ),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekend_only_due_saturday_jumps_to_prev_saturday() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Sat May 23, Day2 WeekendOnly:
        //   Fri→skip … Sun 17→1, Sat 16→0
        //   attention = Sat May 16 → ThisWeek (16 <= 17)
        assert_eq!(
            group(
                "2026-05-23",
                Some(Est::Day2),
                Avail::WeekendOnly,
                "2026-05-11"
            ),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekend_only_due_sunday() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Sun May 24, Day2 WeekendOnly:
        //   Sat 23→1, Fri→skip … Sun 17→0
        //   attention = Sun May 17 → ThisWeek (17 <= 17)
        assert_eq!(
            group(
                "2026-05-24",
                Some(Est::Day2),
                Avail::WeekendOnly,
                "2026-05-11"
            ),
            DeadlineGroup::ThisWeek,
        );
    }

    #[test]
    fn day2_weekend_only_due_monday_jumps_far_back() {
        // Today Mon May 11 (this_sun=17, next_sun=24)
        // Due Mon May 25, Day2 WeekendOnly:
        //   Sun 24→1, Sat 23→0
        //   attention = Sat May 23 → NextWeek (23 <= 24)
        assert_eq!(
            group(
                "2026-05-25",
                Some(Est::Day2),
                Avail::WeekendOnly,
                "2026-05-11"
            ),
            DeadlineGroup::NextWeek,
        );
    }

    // ── Due today with estimate ──

    #[test]
    fn due_today_with_day2_estimate() {
        // Due today → not overdue. Attention has passed → Today.
        assert_eq!(
            group("2026-05-09", Some(Est::Day2), Avail::Anytime, "2026-05-09"),
            DeadlineGroup::Today,
        );
    }

    #[test]
    fn due_today_with_day1_weekend_only() {
        // Due Sat May 9 (today). Not overdue.
        // Day1 WeekendOnly: attention = previous Sun May 3.
        // group_date <= today → Today.
        assert_eq!(
            group(
                "2026-05-09",
                Some(Est::Day1),
                Avail::WeekendOnly,
                "2026-05-09"
            ),
            DeadlineGroup::Today,
        );
    }

    // ── display_label: weekend-aware labels ──

    #[test]
    fn display_label_this_weekend() {
        assert_eq!(DeadlineGroup::ThisWeek.display_label(true), "This Weekend",);
    }

    #[test]
    fn display_label_next_weekend() {
        assert_eq!(DeadlineGroup::NextWeek.display_label(true), "Next Weekend",);
    }

    #[test]
    fn display_label_this_week_when_not_all_weekend() {
        assert_eq!(DeadlineGroup::ThisWeek.display_label(false), "This Week",);
    }

    #[test]
    fn display_label_next_week_when_not_all_weekend() {
        assert_eq!(DeadlineGroup::NextWeek.display_label(false), "Next Week",);
    }

    #[test]
    fn display_label_other_groups_ignore_weekend_flag() {
        // all_weekend=true must not affect non-week groups.
        assert_eq!(DeadlineGroup::Overdue.display_label(true), "Overdue");
        assert_eq!(DeadlineGroup::Today.display_label(true), "Today");
        assert_eq!(DeadlineGroup::Tomorrow.display_label(true), "Tomorrow");
        assert_eq!(DeadlineGroup::Later.display_label(true), "Later");
        assert_eq!(
            DeadlineGroup::ReadyToStart.display_label(true),
            "Ready to Start",
        );
    }
}

// ── End-to-end tests for group_upcoming ─────────────────────────
//
// Each scenario creates a minimal TaskCache and asserts the full
// output of group_upcoming: group order, task order within groups,
// urgency levels, and backlog separation.

#[cfg(all(test, feature = "ssr"))]
mod upcoming_tests {
    use super::*;
    use chrono::NaiveDate;
    use kid_types::server::TaskCache;
    use kid_types::{
        TaskAvailability as Avail, TaskTimeEstimate as Est,
        task::{Date, Summary, Task},
    };

    fn date(today: &str) -> NaiveDate {
        NaiveDate::parse_from_str(today, "%Y-%m-%d").unwrap()
    }

    fn task_date(s: &str) -> Date {
        let nd = date(s);
        Date {
            date: nd.and_hms_opt(0, 0, 0).unwrap().and_utc().fixed_offset(),
            soft: false,
        }
    }

    fn add(
        cache: &mut TaskCache,
        name: &str,
        due: Option<&str>,
        start: Option<&str>,
        estimate: Option<Est>,
        avail: Avail,
    ) -> Uuid {
        use kid_types::TaskDetails;
        let mut t = Task::new(name.parse::<Summary>().unwrap());
        if let Some(d) = due {
            t.set_due_date(task_date(d));
        }
        if let Some(s) = start {
            t.set_start_date(task_date(s));
        }
        if let Some(e) = estimate {
            t.set_time_estimate(e);
        }
        t.set_availability(avail);
        cache.add(t, "test")
    }

    /// Flatten groups into `(group, summary, urgency)` for easy asserts.
    fn flatten(
        groups: &[(
            DeadlineGroup,
            Vec<(Uuid, kid_types::task::Infos, Urgency, NaiveDate)>,
        )],
    ) -> Vec<(DeadlineGroup, &str, Urgency)> {
        groups
            .iter()
            .flat_map(|(dg, tasks)| {
                tasks
                    .iter()
                    .map(move |(_, info, urg, _)| (*dg, info.summary(), *urg))
            })
            .collect()
    }

    /// Just the group labels in order.
    fn group_names(
        groups: &[(
            DeadlineGroup,
            Vec<(Uuid, kid_types::task::Infos, Urgency, NaiveDate)>,
        )],
    ) -> Vec<DeadlineGroup> {
        groups.iter().map(|(dg, _)| *dg).collect()
    }

    /// Just the summaries in order.
    fn summaries(
        groups: &[(
            DeadlineGroup,
            Vec<(Uuid, kid_types::task::Infos, Urgency, NaiveDate)>,
        )],
    ) -> Vec<&str> {
        groups
            .iter()
            .flat_map(|(_, tasks)| tasks.iter().map(|(_, info, _, _)| info.summary()))
            .collect()
    }

    /// Display labels mirroring the UI weekend-detection logic.
    fn display_labels(
        groups: &[(
            DeadlineGroup,
            Vec<(Uuid, kid_types::task::Infos, Urgency, NaiveDate)>,
        )],
    ) -> Vec<&'static str> {
        use chrono::{Datelike, Weekday};
        groups
            .iter()
            .map(|(dg, tasks)| {
                let all_weekend = matches!(dg, DeadlineGroup::ThisWeek | DeadlineGroup::NextWeek)
                    && tasks
                        .iter()
                        .all(|(_, _, _, sd)| matches!(sd.weekday(), Weekday::Sat | Weekday::Sun));
                dg.display_label(all_weekend)
            })
            .collect()
    }

    // ── Scenario 1: baseline grouping by due_date ──

    #[test]
    fn baseline_groups_by_due_date() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // 3 tasks: due today, due this week, due later.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "today-task",
            Some("2026-05-11"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "this-week-task",
            Some("2026-05-15"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "later-task",
            Some("2026-05-30"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(backlog.is_empty());
        assert_eq!(
            group_names(&groups),
            vec![
                DeadlineGroup::Today,
                DeadlineGroup::ThisWeek,
                DeadlineGroup::Later
            ],
        );
        assert_eq!(summaries(&groups).len(), 3);
    }

    // ── Scenario 1b: baseline with Tomorrow ──

    #[test]
    fn baseline_with_tomorrow() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // 4 tasks: today, tomorrow, this week, later.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "today-task",
            Some("2026-05-11"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "tomorrow-task",
            Some("2026-05-12"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "this-week-task",
            Some("2026-05-15"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "later-task",
            Some("2026-05-30"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(
            group_names(&groups),
            vec![
                DeadlineGroup::Today,
                DeadlineGroup::Tomorrow,
                DeadlineGroup::ThisWeek,
                DeadlineGroup::Later,
            ],
        );
        assert_eq!(
            summaries(&groups),
            vec![
                "today-task",
                "tomorrow-task",
                "this-week-task",
                "later-task",
            ]
        );
    }

    // ── Scenario 1c: Tomorrow on Sunday crosses ISO week ──

    #[test]
    fn tomorrow_on_sunday_crosses_iso_week() {
        // Today: Sun May 10 (this_sun=10, next_sun=17)
        // Due Mon May 11 = tomorrow → Tomorrow (not NextWeek)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "monday-task",
            Some("2026-05-11"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-10"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, DeadlineGroup::Tomorrow);
    }

    // ── Scenario 1d: attention date lands on Tomorrow ──

    #[test]
    fn estimate_pulls_to_tomorrow() {
        // Today: Tue May 13 (this_sun=17, next_sun=24)
        // Due Fri May 16, Day2 Anytime:
        //   attention = May 14 (Wed) = tomorrow → Tomorrow
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "shifted-to-tomorrow",
            Some("2026-05-16"),
            None,
            Some(Est::Day2),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-13"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Tomorrow);
        // Day2 → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 2: estimate shifts task to earlier group ──

    #[test]
    fn estimate_pulls_task_forward() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Task due Tue May 19, Day2 Anytime:
        //   attention = May 17 (Sun) → This Week
        // Without estimate it would be Next Week.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "shifted",
            Some("2026-05-19"),
            None,
            Some(Est::Day2),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Day2 → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 3: HalfDay estimate → no group shift, High urgency ──

    #[test]
    fn halfday_estimate_no_shift() {
        // Today: Mon May 11. Due Tue May 19, HalfDay.
        // lead_days=0 → group_date = due_date → Next Week.
        // HalfDay ≥ HalfDay → Urgency::High.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "quick",
            Some("2026-05-19"),
            None,
            Some(Est::HalfDay),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::NextWeek);
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 4: weekend availability skips weekdays ──

    #[test]
    fn weekend_only_skips_weekdays() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due Fri May 22, Day2 WeekendOnly:
        //   count 2 weekend days back: Sun 17→1, Sat 16→0
        //   attention = Sat May 16 → This Week
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "weekend-task",
            Some("2026-05-22"),
            None,
            Some(Est::Day2),
            Avail::WeekendOnly,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Day2 → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 5: backlog — no dates at all ──

    #[test]
    fn task_without_dates_goes_to_backlog() {
        let mut cache = TaskCache::default();
        add(&mut cache, "backlog-task", None, None, None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(groups.is_empty());
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog[0].1.summary(), "backlog-task");
    }

    // ── Scenario 6: start_date without due_date → Ready to Start ──

    #[test]
    fn start_date_only_ready_to_start() {
        // start_date = today, no due_date → ReadyToStart
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "started",
            None,
            Some("2026-05-11"),
            None,
            Avail::Anytime,
        );

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(backlog.is_empty());
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ReadyToStart);
        // ReadyToStart → Urgency::None (no deadline pressure)
        assert_eq!(flat[0].2, Urgency::None);
    }

    // ── Scenario 7: start_date in future, no due_date → backlog ──

    #[test]
    fn future_start_date_no_due_is_backlog() {
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "not-yet",
            None,
            Some("2026-05-20"),
            None,
            Avail::Anytime,
        );

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(groups.is_empty());
        assert_eq!(backlog.len(), 1);
    }

    // ── Scenario 8: start_date pulls task into earlier group (soft) ──

    #[test]
    fn start_date_earlier_than_attention_pulls_forward_soft() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Task: due May 30 (Later), no estimate → attention = May 30.
        // start_date = May 14 (This Week).
        // start_date < attention → group by start_date → This Week, soft.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "early-start",
            Some("2026-05-30"),
            Some("2026-05-14"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // soft + no estimate → Urgency::Standard
        assert_eq!(flat[0].2, Urgency::Standard);
    }

    // ── Scenario 9: soft tasks sort after hard tasks in same group ──

    #[test]
    fn soft_tasks_sort_after_hard_in_same_group() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        //
        // Hard task: due Tue May 19, Day2 Anytime →
        //   attention = May 17 → This Week (hard, attention-driven)
        // Soft task: due May 30, start_date May 14 →
        //   attention = May 30, but start_date pulls to This Week (soft)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "hard-attention",
            Some("2026-05-19"),
            None,
            Some(Est::Day2),
            Avail::Anytime,
        );
        add(
            &mut cache,
            "soft-start",
            Some("2026-05-30"),
            Some("2026-05-14"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let names = summaries(&groups);
        // Hard before soft within the same group.
        assert_eq!(names, vec!["hard-attention", "soft-start"]);
        // Both in This Week.
        assert_eq!(group_names(&groups), vec![DeadlineGroup::ThisWeek]);
    }

    // ── Scenario 10: start_date doesn't shift when same group ──

    #[test]
    fn start_date_same_group_no_shift() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due May 15 (This Week), start May 13 (also This Week).
        // start < due but same group → no soft shift.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "same-group",
            Some("2026-05-15"),
            Some("2026-05-13"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
    }

    // ── Scenario 11: overdue ignores start_date ──

    #[test]
    fn overdue_ignores_start_date() {
        // Due yesterday. start_date doesn't rescue from Overdue.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "overdue",
            Some("2026-05-10"),
            Some("2026-05-01"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Overdue);
        // Overdue → always Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 12: done tasks excluded ──

    #[test]
    fn done_tasks_excluded() {
        let mut cache = TaskCache::default();
        let id = add(
            &mut cache,
            "done-task",
            Some("2026-05-15"),
            None,
            None,
            Avail::Anytime,
        );
        {
            let mut guard = cache.get_mut(&id, "test").unwrap();
            guard.mark_done();
        }

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert!(groups.is_empty());
        assert!(backlog.is_empty());
    }

    // ── Scenario 13: mixed — overdue, today, shifted, backlog ──

    #[test]
    fn mixed_scenario() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "overdue",
            Some("2026-05-10"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "today",
            Some("2026-05-11"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "shifted-to-this-week",
            Some("2026-05-19"),
            None,
            Some(Est::Day2),
            Avail::Anytime,
        );
        add(&mut cache, "backlog", None, None, None, Avail::Anytime);

        let (groups, backlog) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(
            group_names(&groups),
            vec![
                DeadlineGroup::Overdue,
                DeadlineGroup::Today,
                DeadlineGroup::ThisWeek
            ],
        );
        assert_eq!(backlog.len(), 1);
        assert_eq!(backlog[0].1.summary(), "backlog");
    }

    // ── Scenario 14: start_date in past pulls to Today (soft) ──

    #[test]
    fn past_start_date_pulls_to_today_soft() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due May 30 (Later), start May 5 (past).
        // start < attention (May 30) → start group = Today (soft).
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "started-last-week",
            Some("2026-05-30"),
            Some("2026-05-05"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Today);
        // soft + no estimate → Urgency::Standard
        assert_eq!(flat[0].2, Urgency::Standard);
    }

    // ── Scenario 15: soft Today sorts after hard Today ──

    #[test]
    fn soft_today_after_hard_today() {
        // Today: Mon May 11
        // Hard: due May 11 (Today, naturally)
        // Soft: due May 30, start May 05 (Today via start_date)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "hard-today",
            Some("2026-05-11"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "soft-today",
            Some("2026-05-30"),
            Some("2026-05-05"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let names = summaries(&groups);
        assert_eq!(names, vec!["hard-today", "soft-today"]);
    }

    // ── Scenario 16: start_date >= attention → no soft shift ──

    #[test]
    fn start_date_after_attention_no_shift() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due Tue May 19, Day2 → attention May 17 (This Week).
        // start May 18 (after attention) → no soft shift, stays This Week.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "late-start",
            Some("2026-05-19"),
            Some("2026-05-18"),
            Some(Est::Day2),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Day2, not soft → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 17: WeekdayOnly shifts group ──

    #[test]
    fn weekday_only_skips_weekend() {
        // Today: Wed May 13 (tomorrow=14, this_sun=17, next_sun=24)
        // Due Mon May 18, Day2 WeekdayOnly:
        //   Sun 17→skip, Sat 16→skip, Fri 15→1, Thu 14→0
        //   attention = Thu May 14 = tomorrow → Tomorrow
        // Without estimate it would be Next Week (18 > 17).
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "weekday-task",
            Some("2026-05-18"),
            None,
            Some(Est::Day2),
            Avail::WeekdayOnly,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-13"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Tomorrow);
        // Day2, not soft → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 18: WeekdayOnly Day1 ──

    #[test]
    fn weekday_only_day1_skips_weekend() {
        // Today: Wed May 13 (this_sun=17, next_sun=24)
        // Due Mon May 18, Day1 WeekdayOnly:
        //   Sun 17→skip, Sat 16→skip, Fri 15→0
        //   attention = Fri May 15 → This Week
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "weekday-1d",
            Some("2026-05-18"),
            None,
            Some(Est::Day1),
            Avail::WeekdayOnly,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-13"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Day1, not soft → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 19: availability + start_date interaction ──

    #[test]
    fn weekend_only_with_earlier_start_date() {
        // Today: Mon May 11 (this_sun=17, next_sun=24)
        // Due Fri May 22, Day2 WeekendOnly:
        //   attention = Sat May 16 → This Week
        // start_date = May 12 (Mon) < attention (May 16)
        //   start group = This Week (12 <= 17)
        //   Same group → no soft shift, stays hard This Week.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "weekend-with-start",
            Some("2026-05-22"),
            Some("2026-05-12"),
            Some(Est::Day2),
            Avail::WeekendOnly,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::ThisWeek);
        // Day2, not soft (start didn't pull earlier) → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 20: WeekendOnly + past start_date on weekday ──
    //    must NOT be pulled into Today

    #[test]
    fn weekend_only_past_start_date_on_weekday_not_today() {
        // Today: Wed May 20 (this_sun=24, next_sun=31)
        // Due Sun May 31, Day2 WeekendOnly:
        //   count 2 weekend days back from May 31 (Sun):
        //     Sat 30 → 1, Fri→skip … Sun 24 → 0
        //   attention = Sun May 24 → ThisWeek (24 <= this_sun 24)
        //
        // start_date = Mon May 18 (past, weekday).
        // BUG: raw sd (May 18) <= today → start_group = Today,
        //   Today(1) < ThisWeek(2) → task incorrectly pulled to Today.
        //
        // EXPECTED: The start_date override should respect availability.
        //   Next eligible day >= today for WeekendOnly = Sat May 23.
        //   May 23 → ThisWeek (23 <= 24). Same group as attention →
        //   no shift. Task stays in ThisWeek.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "weekend-task-not-today",
            Some("2026-05-31"),
            Some("2026-05-18"),
            Some(Est::Day2),
            Avail::WeekendOnly,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-20"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(
            flat[0].0,
            DeadlineGroup::ThisWeek,
            "WeekendOnly task must not land in Today on a Wednesday; \
             next eligible day is Sat May 23 → ThisWeek",
        );
        // Day2, not soft → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── Scenario 21: WeekdayOnly + past start_date on weekend ──
    //    symmetric counterpart to Scenario 20

    #[test]
    fn weekday_only_past_start_date_on_weekend_not_today() {
        // Today: Sat May 9 (this_sun=10, next_sun=17)
        // Due Thu May 14, Day2 WeekdayOnly:
        //   count 2 weekdays back from May 14 (Thu):
        //     Wed 13 → 1, Tue 12 → 0
        //   attention = Tue May 12 → NextWeek (12 > 10, 12 <= 17)
        //
        // start_date = Mon May 5 (past, weekday — eligible, but
        //   today is Saturday which is NOT eligible).
        //
        // Without fix: sd (May 5) <= today → Today,
        //   Today(1) < NextWeek(3) → incorrectly pulled to Today
        //   on a Saturday.
        //
        // With fix: eff = max(5, 9) = Sat 9 → not eligible →
        //   Sun 10 → not eligible → Mon 11 ✓
        //   Mon 11 → NextWeek (11 > 10, 11 <= 17).
        //   NextWeek == NextWeek → no shift. Stays NextWeek.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "weekday-task-not-today",
            Some("2026-05-14"),
            Some("2026-05-05"),
            Some(Est::Day2),
            Avail::WeekdayOnly,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-09"));
        let flat = flatten(&groups);
        assert_eq!(flat.len(), 1);
        assert_eq!(
            flat[0].0,
            DeadlineGroup::NextWeek,
            "WeekdayOnly task must not land in Today on a Saturday; \
             next eligible day is Mon May 11 → NextWeek",
        );
        // Day2, not soft → Urgency::High
        assert_eq!(flat[0].2, Urgency::High);
    }

    // ── display_label integration: weekend-aware labels ──

    #[test]
    fn this_week_all_weekend_shows_weekend_label() {
        // Today: Mon May 11 (this_sun=17). Tasks due Sat 16 + Sun 17.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "sat-task",
            Some("2026-05-16"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "sun-task",
            Some("2026-05-17"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["This Weekend"]);
    }

    #[test]
    fn this_week_single_weekend_task_shows_weekend_label() {
        // Today: Mon May 11. Solo task due Sat 16.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "solo-sat",
            Some("2026-05-16"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["This Weekend"]);
    }

    #[test]
    fn this_week_mixed_keeps_week_label() {
        // Today: Mon May 11. Tasks due Wed 13 + Sat 16.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "wed-task",
            Some("2026-05-13"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "sat-task",
            Some("2026-05-16"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["This Week"]);
    }

    #[test]
    fn this_week_all_weekday_keeps_week_label() {
        // Today: Mon May 11. Tasks due Wed 13 + Thu 14.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "wed-task",
            Some("2026-05-13"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "thu-task",
            Some("2026-05-14"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["This Week"]);
    }

    #[test]
    fn next_week_all_weekend_shows_weekend_label() {
        // Today: Mon May 11 (next_sun=24). Tasks due Sat 23 + Sun 24.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "next-sat",
            Some("2026-05-23"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "next-sun",
            Some("2026-05-24"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["Next Weekend"]);
    }

    #[test]
    fn next_week_mixed_keeps_week_label() {
        // Today: Mon May 11. Tasks due Tue 19 + Sat 23.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "next-tue",
            Some("2026-05-19"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "next-sat",
            Some("2026-05-23"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["Next Week"]);
    }

    #[test]
    fn multiple_groups_weekend_labels_independent() {
        // Today: Mon May 11. ThisWeek: only Sat 16 → Weekend.
        // NextWeek: Tue 19 + Sat 23 → mixed → Week.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "this-sat",
            Some("2026-05-16"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "next-tue",
            Some("2026-05-19"),
            None,
            None,
            Avail::Anytime,
        );
        add(
            &mut cache,
            "next-sat",
            Some("2026-05-23"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        assert_eq!(display_labels(&groups), vec!["This Weekend", "Next Week"],);
    }

    // ── Urgency levels (UXDR: upcoming-urgency-checkbox) ──

    #[test]
    fn urgency_sub_day_estimate_is_hard() {
        // Due today, Min30 → Hard (completable in a gap)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "quick",
            Some("2026-05-11"),
            None,
            Some(Est::Min30),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].2, Urgency::Hard);
    }

    #[test]
    fn urgency_hours2_is_hard() {
        // Due today, Hours2 → Hard (still sub-day, grabs a gap)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "two-hours",
            Some("2026-05-11"),
            None,
            Some(Est::Hours2),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].2, Urgency::Hard);
    }

    #[test]
    fn urgency_halfday_is_high() {
        // Due today, HalfDay → High (needs dedicated time)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "half-day",
            Some("2026-05-11"),
            None,
            Some(Est::HalfDay),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].2, Urgency::High);
    }

    #[test]
    fn urgency_no_estimate_is_high() {
        // Due today, no estimate → High (scope unknown)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "unknown",
            Some("2026-05-11"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].2, Urgency::High);
    }

    #[test]
    fn urgency_overdue_sub_day_is_hard() {
        // Overdue with Min15 estimate → Hard (quick to knock out)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "overdue-quick",
            Some("2026-05-10"),
            None,
            Some(Est::Min15),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Overdue);
        assert_eq!(flat[0].2, Urgency::Hard);
    }

    #[test]
    fn urgency_overdue_no_estimate_is_high() {
        // Overdue with no estimate → High (scope unknown)
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "overdue-unknown",
            Some("2026-05-10"),
            None,
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Overdue);
        assert_eq!(flat[0].2, Urgency::High);
    }

    #[test]
    fn urgency_soft_sub_day_is_none() {
        // Due May 30 (Later), start May 5 (past), Min30 estimate.
        // Soft + sub-day → Urgency::None.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "soft-quick",
            Some("2026-05-30"),
            Some("2026-05-05"),
            Some(Est::Min30),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Today);
        assert_eq!(flat[0].2, Urgency::None);
    }

    #[test]
    fn urgency_soft_halfday_is_standard() {
        // Due May 30 (Later), start May 5 (past), HalfDay estimate.
        // Soft + ≥ HalfDay → Urgency::Standard.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "soft-big",
            Some("2026-05-30"),
            Some("2026-05-05"),
            Some(Est::HalfDay),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Today);
        assert_eq!(flat[0].2, Urgency::Standard);
    }

    #[test]
    fn urgency_soft_no_estimate_is_standard() {
        // Due May 30 (Later), start May 5 (past), no estimate.
        // Soft + unknown scope → Urgency::Standard.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "soft-unknown",
            Some("2026-05-30"),
            Some("2026-05-05"),
            None,
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Today);
        assert_eq!(flat[0].2, Urgency::Standard);
    }

    #[test]
    fn urgency_later_always_none() {
        // Due in 30 days (Later), Day2 estimate.
        // Would be High if not in Later, but Later suppresses urgency.
        let mut cache = TaskCache::default();
        add(
            &mut cache,
            "far-away",
            Some("2026-06-10"),
            None,
            Some(Est::Day2),
            Avail::Anytime,
        );

        let (groups, _) = group_upcoming(cache.iter(), date("2026-05-11"));
        let flat = flatten(&groups);
        assert_eq!(flat[0].0, DeadlineGroup::Later);
        assert_eq!(flat[0].2, Urgency::None);
    }
}

// ── Quick Wins: intra-group sort tests ─────────────────────────

#[cfg(all(test, feature = "ssr"))]
mod quick_wins_tests {
    use super::*;
    use kid_types::server::TaskCache;
    use kid_types::{
        TaskDetails, TaskInfos, TaskPriority as Pri, TaskTimeEstimate as Est,
        task::{Summary, Task},
    };

    fn add(cache: &mut TaskCache, name: &str, estimate: Est, priority: Option<Pri>) -> Uuid {
        let mut t = Task::new(name.parse::<Summary>().unwrap());
        t.set_time_estimate(estimate);
        if let Some(p) = priority {
            t.set_priority(p);
        }
        cache.add(t, "test")
    }

    /// Flat list of `(estimate, summary)` pairs in output order.
    fn flatten(groups: &[(Est, Vec<(Uuid, task::Infos)>)]) -> Vec<(Est, &str)> {
        groups
            .iter()
            .flat_map(|(est, tasks)| tasks.iter().map(move |(_, info)| (*est, info.summary())))
            .collect()
    }

    #[test]
    fn priority_ordering_within_estimate_group() {
        let mut cache = TaskCache::default();
        // Insert in reverse priority order.
        add(&mut cache, "low", Est::Min30, Some(Pri::C));
        add(&mut cache, "high", Est::Min30, Some(Pri::A));
        add(&mut cache, "medium", Est::Min30, Some(Pri::B));

        let groups = group_quick_wins(cache.iter());
        assert_eq!(groups.len(), 1);
        let names: Vec<_> = groups[0].1.iter().map(|(_, i)| i.summary()).collect();
        assert_eq!(names, vec!["high", "medium", "low"]);
    }

    #[test]
    fn unprioritized_sorts_after_prioritized() {
        let mut cache = TaskCache::default();
        add(&mut cache, "no-pri", Est::Min15, None);
        add(&mut cache, "pri-a", Est::Min15, Some(Pri::A));

        let groups = group_quick_wins(cache.iter());
        let names: Vec<_> = groups[0].1.iter().map(|(_, i)| i.summary()).collect();
        assert_eq!(names, vec!["pri-a", "no-pri"]);
    }

    #[test]
    fn uuid_tiebreak_for_same_priority() {
        let mut cache = TaskCache::default();
        // Same estimate, same priority → older (lower UUID) first.
        add(&mut cache, "first", Est::Hours1, Some(Pri::B));
        add(&mut cache, "second", Est::Hours1, Some(Pri::B));

        let groups = group_quick_wins(cache.iter());
        let names: Vec<_> = groups[0].1.iter().map(|(_, i)| i.summary()).collect();
        assert_eq!(names, vec!["first", "second"]);
    }

    #[test]
    fn groups_ordered_by_estimate_ascending() {
        let mut cache = TaskCache::default();
        // Insert out of order.
        add(&mut cache, "big", Est::Day2, None);
        add(&mut cache, "tiny", Est::Min15, None);
        add(&mut cache, "medium", Est::Hours1, None);

        let groups = group_quick_wins(cache.iter());
        let estimates: Vec<_> = groups.iter().map(|(e, _)| *e).collect();
        assert_eq!(estimates, vec![Est::Min15, Est::Hours1, Est::Day2]);
    }

    #[test]
    fn done_tasks_excluded() {
        let mut cache = TaskCache::default();
        let id = add(&mut cache, "done", Est::Min30, None);
        {
            let mut guard = cache.get_mut(&id, "test").unwrap();
            guard.mark_done();
        }

        let groups = group_quick_wins(cache.iter());
        assert!(groups.is_empty());
    }

    #[test]
    fn tasks_without_estimate_excluded() {
        let mut cache = TaskCache::default();
        let t = Task::new("no-estimate".parse::<Summary>().unwrap());
        cache.add(t, "test");

        let groups = group_quick_wins(cache.iter());
        assert!(groups.is_empty());
    }

    #[test]
    fn mixed_estimates_and_priorities() {
        let mut cache = TaskCache::default();
        add(&mut cache, "m30-b", Est::Min30, Some(Pri::B));
        add(&mut cache, "m30-a", Est::Min30, Some(Pri::A));
        add(&mut cache, "m15", Est::Min15, None);
        add(&mut cache, "h1-c", Est::Hours1, Some(Pri::C));
        add(&mut cache, "h1-a", Est::Hours1, Some(Pri::A));

        let groups = group_quick_wins(cache.iter());
        assert_eq!(
            flatten(&groups),
            vec![
                (Est::Min15, "m15"),
                (Est::Min30, "m30-a"),
                (Est::Min30, "m30-b"),
                (Est::Hours1, "h1-a"),
                (Est::Hours1, "h1-c"),
            ],
        );
    }
}

// ── What I Finished: intra-group sort tests ────────────────────

#[cfg(all(test, feature = "ssr"))]
mod finished_tests {
    use super::*;
    use kid_types::server::TaskCache;
    use kid_types::{
        TaskCategory, TaskInfos,
        task::{Summary, Task},
    };
    use std::time::Duration;

    fn add_done(cache: &mut TaskCache, name: &str, category: &str) -> Uuid {
        let mut t = Task::new(name.parse::<Summary>().unwrap());
        t.set_category(category.parse().unwrap());
        let id = cache.add(t, "test");
        {
            let mut guard = cache.get_mut(&id, "test").unwrap();
            guard.mark_done();
        }
        id
    }

    fn cat(s: &str) -> TaskCategory {
        s.parse().unwrap()
    }

    #[test]
    fn completion_order_within_category() {
        let mut cache = TaskCache::default();
        // Mark done in order: first → second → third.
        // Most recently completed should appear first.
        add_done(&mut cache, "first", "Work");
        std::thread::sleep(Duration::from_millis(10));
        add_done(&mut cache, "second", "Work");
        std::thread::sleep(Duration::from_millis(10));
        add_done(&mut cache, "third", "Work");

        let groups = group_finished(cache.iter());
        let work = groups.get(&cat("Work")).unwrap();
        let names: Vec<_> = work.iter().map(|(_, info)| info.summary()).collect();
        assert_eq!(names, vec!["third", "second", "first"]);
    }

    #[test]
    fn categories_in_alphabetical_order() {
        let mut cache = TaskCache::default();
        add_done(&mut cache, "z-task", "Zzz");
        add_done(&mut cache, "a-task", "Aaa");
        add_done(&mut cache, "m-task", "Mmm");

        let groups = group_finished(cache.iter());
        let cats: Vec<_> = groups.keys().map(|c| c.to_string()).collect();
        assert_eq!(cats, vec!["Aaa", "Mmm", "Zzz"]);
    }

    #[test]
    fn independent_sort_per_category() {
        let mut cache = TaskCache::default();
        add_done(&mut cache, "home-a", "Home");
        std::thread::sleep(Duration::from_millis(10));
        add_done(&mut cache, "home-b", "Home");
        std::thread::sleep(Duration::from_millis(10));
        add_done(&mut cache, "work-x", "Work");
        std::thread::sleep(Duration::from_millis(10));
        add_done(&mut cache, "work-y", "Work");

        let groups = group_finished(cache.iter());
        let home: Vec<_> = groups
            .get(&cat("Home"))
            .unwrap()
            .iter()
            .map(|(_, i)| i.summary())
            .collect();
        let work: Vec<_> = groups
            .get(&cat("Work"))
            .unwrap()
            .iter()
            .map(|(_, i)| i.summary())
            .collect();
        assert_eq!(home, vec!["home-b", "home-a"]);
        assert_eq!(work, vec!["work-y", "work-x"]);
    }

    #[test]
    fn open_tasks_excluded() {
        let mut cache = TaskCache::default();
        let t = Task::new("open".parse::<Summary>().unwrap());
        cache.add(t, "test");

        let groups = group_finished(cache.iter());
        assert!(groups.is_empty());
    }
}

// ── All Open: grouping and sort tests ──────────────────────────

#[cfg(all(test, feature = "ssr"))]
mod all_open_tests {
    use super::*;
    use kid_types::server::TaskCache;
    use kid_types::{
        TaskCategory, TaskInfos,
        task::{Summary, Task},
    };

    fn add(cache: &mut TaskCache, name: &str, category: &str) -> Uuid {
        let mut t = Task::new(name.parse::<Summary>().unwrap());
        t.set_category(category.parse().unwrap());
        cache.add(t, "test")
    }

    fn cat(s: &str) -> TaskCategory {
        s.parse().unwrap()
    }

    #[test]
    fn categories_in_alphabetical_order() {
        let mut cache = TaskCache::default();
        add(&mut cache, "z-task", "Zzz");
        add(&mut cache, "a-task", "Aaa");
        add(&mut cache, "m-task", "Mmm");

        let groups = group_all_open(cache.iter());
        let cats: Vec<_> = groups.keys().map(|c| c.to_string()).collect();
        assert_eq!(cats, vec!["Aaa", "Mmm", "Zzz"]);
    }

    #[test]
    fn uuid_sort_within_category() {
        let mut cache = TaskCache::default();
        // Add three tasks to same category — oldest UUID first.
        let id_a = add(&mut cache, "older", "Work");
        let id_b = add(&mut cache, "newer", "Work");

        let groups = group_all_open(cache.iter());
        let work = groups.get(&cat("Work")).unwrap();
        assert_eq!(work[0].0, id_a);
        assert_eq!(work[1].0, id_b);
    }

    #[test]
    fn done_tasks_excluded() {
        let mut cache = TaskCache::default();
        let id = add(&mut cache, "done", "Work");
        {
            let mut guard = cache.get_mut(&id, "test").unwrap();
            guard.mark_done();
        }

        let groups = group_all_open(cache.iter());
        assert!(groups.is_empty());
    }

    #[test]
    fn mixed_categories_with_uuid_order() {
        let mut cache = TaskCache::default();
        let id_w1 = add(&mut cache, "work-1", "Work");
        let id_h1 = add(&mut cache, "home-1", "Home");
        let id_w2 = add(&mut cache, "work-2", "Work");
        let id_h2 = add(&mut cache, "home-2", "Home");

        let groups = group_all_open(cache.iter());
        // Alphabetical: Home before Work.
        let cats: Vec<_> = groups.keys().map(|c| c.to_string()).collect();
        assert_eq!(cats, vec!["Home", "Work"]);
        // Within each: older UUID first.
        let home = groups.get(&cat("Home")).unwrap();
        assert_eq!(home[0].0, id_h1);
        assert_eq!(home[1].0, id_h2);
        let work = groups.get(&cat("Work")).unwrap();
        assert_eq!(work[0].0, id_w1);
        assert_eq!(work[1].0, id_w2);
    }
}

// ── Recently Changed: sort, AI flags, and pagination tests ─────

#[cfg(all(test, feature = "ssr"))]
mod recently_changed_tests {
    use super::*;
    use kid_types::server::TaskCache;
    use kid_types::{
        TaskInfos, TaskPriority as Pri,
        task::{Summary, Task},
    };
    use std::time::Duration;

    fn today() -> NaiveDate {
        chrono::Utc::now().date_naive()
    }

    // NOTE: add_author() truncates timestamps to whole seconds
    // (with_nanosecond(0)), so sleeps must cross a second boundary
    // to produce distinct author timestamps.

    #[test]
    fn sorted_by_last_changed_descending() {
        let mut cache = TaskCache::default();
        let t = Task::new("first".parse::<Summary>().unwrap());
        cache.add(t, "a");
        std::thread::sleep(Duration::from_secs(1));
        let t = Task::new("second".parse::<Summary>().unwrap());
        cache.add(t, "b");
        std::thread::sleep(Duration::from_secs(1));
        let t = Task::new("third".parse::<Summary>().unwrap());
        cache.add(t, "c");

        let result = group_recently_changed(cache.iter(), today(), 0);
        let names: Vec<_> = result.iter().map(|rc| rc.info.summary()).collect();
        assert_eq!(names, vec!["third", "second", "first"]);
    }

    #[test]
    fn ai_involved_and_last_when_ai_edited() {
        let mut cache = TaskCache::default();
        let t = Task::new("task".parse::<Summary>().unwrap());
        let id = cache.add(t, "human");
        std::thread::sleep(Duration::from_secs(1));
        {
            let mut guard = cache.get_mut(&id, "ai:bot").unwrap();
            guard.set_priority(Pri::A);
        }

        let result = group_recently_changed(cache.iter(), today(), 0);
        assert_eq!(result.len(), 1);
        assert!(result[0].ai_involved, "ai:bot should set ai_involved");
        assert!(result[0].ai_last, "ai:bot was last actor");
    }

    #[test]
    fn ai_not_last_when_human_acts_after() {
        let mut cache = TaskCache::default();
        let t = Task::new("task".parse::<Summary>().unwrap());
        let id = cache.add(t, "human");
        std::thread::sleep(Duration::from_secs(1));
        {
            let mut guard = cache.get_mut(&id, "ai:bot").unwrap();
            guard.set_priority(Pri::A);
        }
        std::thread::sleep(Duration::from_secs(1));
        {
            let mut guard = cache.get_mut(&id, "human").unwrap();
            guard.set_priority(Pri::B);
        }

        let result = group_recently_changed(cache.iter(), today(), 0);
        assert!(result[0].ai_involved, "ai:bot touched the task");
        assert!(!result[0].ai_last, "human acted after ai:bot");
    }

    #[test]
    fn no_ai_flags_for_human_only() {
        let mut cache = TaskCache::default();
        let t = Task::new("task".parse::<Summary>().unwrap());
        cache.add(t, "human");

        let result = group_recently_changed(cache.iter(), today(), 0);
        assert!(!result[0].ai_involved);
        assert!(!result[0].ai_last);
    }

    #[test]
    fn calendar_window_includes_today() {
        let mut cache = TaskCache::default();
        let t = Task::new("recent".parse::<Summary>().unwrap());
        cache.add(t, "test");

        // today = actual today → task falls within 3-day window.
        let result = group_recently_changed(cache.iter(), today(), 0);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn extra_days_zero_excludes_old_tasks() {
        let mut cache = TaskCache::default();
        let t = Task::new("old".parse::<Summary>().unwrap());
        cache.add(t, "test");

        // Shift "today" 10 days forward → task falls outside
        // the 3-day window.
        let future = today() + chrono::Days::new(10);
        let result = group_recently_changed(cache.iter(), future, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn extra_days_includes_older_data() {
        let mut cache = TaskCache::default();
        let t = Task::new("old".parse::<Summary>().unwrap());
        cache.add(t, "test");

        let future = today() + chrono::Days::new(10);
        // extra_days=1 → picks up one older day with data.
        let result = group_recently_changed(cache.iter(), future, 1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].info.summary(), "old");
    }
}
