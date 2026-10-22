//! Which target a run asks about next: one lane per host, each host holding at
//! most its share of the run's slots, the lanes taken in turn (D61).
//!
//! A host's own pace is the fetcher's (D8), so a task waiting for it holds a
//! slot and does nothing. Without a share, a host paced at one a second fills
//! every slot with waiters and the other hosts run after it.

use std::collections::{HashMap, VecDeque};

use url::Url;

/// What [`Schedule`] groups targets by: the name the fetcher keys a host's
/// pace by, ports and schemes sharing it (`fetch::http::host`).
pub(super) fn host_of(url: &Url) -> String {
    url.host_str().unwrap_or("").to_ascii_lowercase()
}

/// The targets of a run waiting to start, and how many of each host are
/// running.
pub(super) struct Schedule<T> {
    slots: usize,
    running: usize,
    queued: usize,
    lanes: HashMap<String, Lane<T>>,
    /// Every lane, the next to be taken first.
    turn: VecDeque<String>,
}

struct Lane<T> {
    waiting: VecDeque<T>,
    running: usize,
}

impl<T> Schedule<T> {
    /// A schedule of at most `slots` targets running at once.
    pub(super) fn new(slots: usize) -> Schedule<T> {
        Schedule {
            slots,
            running: 0,
            queued: 0,
            lanes: HashMap::new(),
            turn: VecDeque::new(),
        }
    }

    /// `target`, waiting behind the earlier ones of `host`.
    pub(super) fn push(&mut self, host: String, target: T) {
        if !self.lanes.contains_key(&host) {
            self.turn.push_back(host.clone());
        }
        let lane = self.lanes.entry(host).or_insert_with(|| Lane {
            waiting: VecDeque::new(),
            running: 0,
        });
        lane.waiting.push_back(target);
        self.queued += 1;
    }

    /// Whether any target waits.
    pub(super) fn is_waiting(&self) -> bool {
        self.queued > 0
    }

    /// The next target to start, with its host, now counted as running: the
    /// first of the next host in turn that is under its share, a host's share
    /// being the slots divided by the hosts with a target waiting or running
    /// (at least one). `None` when every slot is taken, or every host with a
    /// target waiting holds its share.
    pub(super) fn next(&mut self) -> Option<(String, T)> {
        if self.running >= self.slots {
            return None;
        }
        let share = (self.slots / self.lanes.len().max(1)).max(1);
        for _ in 0..self.turn.len() {
            let host = self.turn.pop_front()?;
            self.turn.push_back(host.clone());
            let lane = self.lanes.get_mut(&host)?;
            if lane.running < share
                && let Some(target) = lane.waiting.pop_front()
            {
                lane.running += 1;
                self.running += 1;
                self.queued -= 1;
                return Some((host, target));
            }
        }
        None
    }

    /// A target of `host` that [`Schedule::next`] returned has ended.
    pub(super) fn finished(&mut self, host: &str) {
        if let Some(lane) = self.lanes.get_mut(host) {
            lane.running -= 1;
            self.running -= 1;
        }
        self.retire(host);
    }

    /// Drops every target not yet started.
    pub(super) fn clear(&mut self) {
        let hosts: Vec<String> = self.lanes.keys().cloned().collect();
        for host in hosts {
            if let Some(lane) = self.lanes.get_mut(&host) {
                lane.waiting.clear();
            }
            self.retire(&host);
        }
        self.queued = 0;
    }

    /// Forgets `host` once it has nothing waiting or running.
    fn retire(&mut self, host: &str) {
        let idle = self
            .lanes
            .get(host)
            .is_some_and(|lane| lane.running == 0 && lane.waiting.is_empty());
        if idle {
            self.lanes.remove(host);
            self.turn.retain(|h| h != host);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started(schedule: &mut Schedule<u32>) -> Vec<(String, u32)> {
        std::iter::from_fn(|| schedule.next()).collect()
    }

    fn fill(schedule: &mut Schedule<u32>, host: &str, targets: std::ops::Range<u32>) {
        for t in targets {
            schedule.push(host.to_owned(), t);
        }
    }

    #[test]
    fn one_host_may_take_every_slot_and_keeps_its_order() {
        let mut s = Schedule::new(4);
        fill(&mut s, "a", 0..10);
        let order: Vec<u32> = started(&mut s).into_iter().map(|(_, t)| t).collect();
        assert_eq!(order, [0, 1, 2, 3]);
        s.finished("a");
        assert_eq!(s.next(), Some(("a".to_owned(), 4)));
    }

    #[test]
    fn a_host_with_a_long_queue_leaves_the_other_its_share_of_the_slots() {
        let mut s = Schedule::new(8);
        fill(&mut s, "slow", 0..100);
        assert_eq!(started(&mut s).len(), 8, "alone, a host has every slot");
        fill(&mut s, "fast", 100..120);
        // The slow host's eight waiters still run; the fast one starts as
        // they end, and is then held to its half, as the slow one is.
        for _ in 0..4 {
            s.finished("slow");
        }
        let next = started(&mut s);
        assert_eq!(
            next.iter().filter(|(h, _)| h == "fast").count(),
            4,
            "the freed slots go to the host holding none: {next:?}"
        );
        assert_eq!(s.next(), None, "every slot is taken");
        s.finished("fast");
        assert_eq!(s.next().map(|(h, _)| h), Some("fast".to_owned()));
        s.finished("slow");
        assert_eq!(s.next().map(|(h, _)| h), Some("slow".to_owned()));
    }

    #[test]
    fn hosts_are_taken_in_turn() {
        let mut s = Schedule::new(6);
        fill(&mut s, "a", 0..3);
        fill(&mut s, "b", 10..13);
        let hosts: Vec<String> = started(&mut s).into_iter().map(|(h, _)| h).collect();
        assert_eq!(hosts, ["a", "b", "a", "b", "a", "b"]);
    }

    #[test]
    fn many_hosts_each_get_a_slot_while_slots_last() {
        let mut s = Schedule::new(3);
        for (i, host) in ["a", "b", "c", "d", "e"].into_iter().enumerate() {
            s.push(host.to_owned(), i as u32);
        }
        assert_eq!(started(&mut s).len(), 3);
        s.finished("a");
        assert_eq!(s.next().map(|(h, _)| h), Some("d".to_owned()));
    }

    #[test]
    fn a_host_with_nothing_waiting_or_running_is_forgotten() {
        let mut s = Schedule::new(4);
        fill(&mut s, "a", 0..1);
        fill(&mut s, "b", 1..2);
        started(&mut s);
        s.finished("a");
        assert_eq!(s.lanes.len(), 1);
        assert_eq!(s.turn.len(), 1);
        s.finished("b");
        assert!(s.lanes.is_empty() && s.turn.is_empty() && !s.is_waiting());
    }

    #[test]
    fn clearing_drops_what_has_not_started_and_keeps_what_runs() {
        let mut s = Schedule::new(2);
        fill(&mut s, "a", 0..5);
        fill(&mut s, "b", 5..10);
        started(&mut s);
        s.clear();
        assert!(!s.is_waiting());
        assert_eq!(s.next(), None);
        s.finished("a");
        s.finished("b");
        assert!(s.lanes.is_empty());
    }

    #[test]
    fn a_host_is_named_by_its_lowercased_name_whatever_its_port() {
        let a = Url::parse("https://Forge.Example:8443/x").unwrap();
        let b = Url::parse("http://forge.example/y").unwrap();
        assert_eq!(host_of(&a), host_of(&b));
    }
}
