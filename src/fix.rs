use regex::Regex;
use std::fs;
use std::sync::{Mutex, MutexGuard};

use crate::common::{BranchCoverage, LineCoverage, PackageCoverage, SourceCode};

struct State {
    is_test: bool,
}

impl State {
    fn new() -> Self {
        State { is_test: false }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

pub struct Fixer {
    ne_reg: Vec<Regex>,
    c_reg: Vec<Regex>,
    ts_reg: Vec<Regex>,
    state: Mutex<State>,
}

impl Fixer {
    pub fn new() -> Self {
        Self {
            ne_reg: vec![
                Regex::new(
                    r"^(?:\s*\}(?:\s*\))*(?:\s*;)?|\s*(?:\}\s*)?else(?:\s*\{)?)?\s*(?://.*)?$",
                )
                .unwrap(),
                Regex::new(r"^\s*pub\s*struct\s*.*?\{\s*(?://.*)?$").unwrap(),
            ],
            c_reg: vec![Regex::new(r"^\s*for\s*.*\{\s*(?://.*)?$").unwrap()],
            ts_reg: vec![Regex::new(r"^\s*mod\s*test\s*\{\s*(?://.*)?$").unwrap()],
            state: Mutex::new(State::new()),
        }
    }

    pub fn fix(&self, data: &mut PackageCoverage) {
        let mut state = self.state.lock().unwrap();

        for file_cov in data.file_coverages_mut() {
            let content = fs::read_to_string(file_cov.path()).unwrap();
            let source = SourceCode::new(content);

            for cov in file_cov.line_coverages_mut() {
                let line = source.get_line(cov.line_number).unwrap();
                self.process_line(line, cov, &mut state);
            }

            state.reset();

            for cov in file_cov.branch_coverages_mut() {
                let line = source.get_line(cov.line_number).unwrap();
                self.process_branch(line, cov, &mut state);
            }

            state.reset();
            file_cov.remove_invalid_coverages();
        }
    }

    fn process_line(&self, line: &str, cov: &mut LineCoverage, state: &mut MutexGuard<State>) {
        if state.is_test {
            return;
        }

        if self.ts_reg.iter().any(|r| r.is_match(line)) {
            state.is_test = true;
            return;
        }

        if self.ne_reg.iter().any(|r| r.is_match(line)) {
            cov.count = std::u32::MAX;
        }

        if cov.count == 0 && self.c_reg.iter().any(|r| r.is_match(line)) {
            cov.count = 1;
        }
    }

    fn process_branch(&self, line: &str, cov: &mut BranchCoverage, state: &mut MutexGuard<State>) {
        if state.is_test {
            return;
        }

        if self.ts_reg.iter().any(|r| r.is_match(line)) {
            state.is_test = true;
            return;
        }

        if !cov.taken && self.c_reg.iter().any(|r| r.is_match(line)) {
            cov.taken = true;
        }
    }
}
