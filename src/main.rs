// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Juntei: Windows forensics artifact parser.

mod amcache;
mod amcache_output;
mod auto_parse;
mod boot;
mod browser;
mod cli;
mod color;
mod defender_quarantine;
mod disk_image;
mod hive;
mod i30;
mod image_files;
mod image_metadata;
mod interrupt;
mod jump_lists;
mod lnk;
mod logfile;
mod mft;
mod mft_csv;
mod mft_inspect;
mod mount;
mod outlook;
mod output_schema;
mod persistence_hunter;
mod prefetch;
mod progress;
mod recycle_bin;
mod registry;
mod secure_sds;
mod security_descriptor;
mod shimcache;
mod srum;
mod super_timeline;
mod time_display;
mod ual;
mod usn_journal;
mod util;
mod webcache;

use clap::{CommandFactory, Parser};
use cli::{
    Cli, Commands, EseCommands, FileCommands, NtfsCommands, RegistryCommands, SqliteCommands,
    VERSION,
};
use color::JunteiColor::Green;
use std::env;
use std::time::Instant;
use util::p;

const LOGO: &str = include_str!("../art/logo.txt");

fn main() {
    let args: Vec<_> = env::args_os().collect();
    if args.len() == 1 || (args.len() == 2 && (args[1] == "-h" || args[1] == "--help")) {
        display_logo(false, false, false, true);
        Cli::command().print_help().ok();
        return;
    }

    let start = Instant::now();
    let cli = Cli::parse();
    let code = match &cli.cmd {
        Commands::SuperTimeline(opts) => {
            display_logo(opts.common.quiet, opts.common.no_color, true, false);
            let code = super_timeline::run(opts);
            print_elapsed(start, opts.common.no_color);
            code
        }
        Commands::PersistenceHunter(opts) => {
            let data_only = opts.data_on_stdout();
            if !data_only && !opts.list_checks {
                display_logo(opts.common.quiet, opts.common.no_color, true, false);
            }
            let code = persistence_hunter::run(opts);
            if !data_only && !opts.list_checks {
                print_elapsed(start, opts.common.no_color);
            }
            code
        }
        Commands::AutoParse(opts) => {
            display_logo(opts.common.quiet, opts.common.no_color, true, false);
            let code = auto_parse::run(opts);
            print_elapsed(start, opts.common.no_color);
            code
        }
        Commands::Mount(opts) => {
            display_logo(opts.common.quiet, opts.common.no_color, true, false);
            let code = mount::run(opts);
            print_elapsed(start, opts.common.no_color);
            code
        }
        Commands::ParseNtfs(source) => match &source.cmd {
            NtfsCommands::Boot(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = boot::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            NtfsCommands::SecureSds(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = secure_sds::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            NtfsCommands::Mft(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = mft::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            NtfsCommands::UsnJournal(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = usn_journal::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            NtfsCommands::Logfile(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = logfile::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            NtfsCommands::I30(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = i30::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
        },
        Commands::ParseSqlite(source) => match &source.cmd {
            SqliteCommands::Browser(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = browser::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
        },
        Commands::ParseEse(source) => match &source.cmd {
            EseCommands::Srum(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = srum::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            EseCommands::Ual(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = ual::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            EseCommands::Webcache(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = webcache::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
        },
        Commands::ParseFile(source) => match &source.cmd {
            FileCommands::Prefetch(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = prefetch::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            FileCommands::Outlook(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = outlook::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            FileCommands::Lnk(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = lnk::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            FileCommands::JumpLists(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = jump_lists::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            FileCommands::RecycleBin(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = recycle_bin::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            FileCommands::DefenderQuarantine(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = defender_quarantine::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
        },
        Commands::ParseRegistry(registry) => match &registry.cmd {
            RegistryCommands::Amcache(opts) => {
                // JSON / JSONL on stdout: nothing but the data is printed there
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = amcache::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Shimcache(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = shimcache::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }

            RegistryCommands::Read(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::read::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Services(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::services::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::RecentDocs(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::recentdocs::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::ShellBags(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::shellbags::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Taskcache(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::taskcache::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Sam(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::sam::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Persistence(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::persistence::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Usb(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::usb::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::UserAssist(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::userassist::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::OpenSaveMru(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::opensavemru::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Bam(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::bam::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::InstalledPrograms(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::installed_programs::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Muicache(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::muicache::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::RemoteAccess(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::remote_access::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::SystemInfo(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::system_info::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Mru(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::mru::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::AppHistory(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::app_history::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::FeatureUsage(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::featureusage::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::SecuritySettings(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::security_settings::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Office(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::office::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Auditpol(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::auditpol::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::AppCompat(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::appcompat::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Bcd(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::bcd::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::LsaSecrets(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::lsa_secrets::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
            RegistryCommands::Networks(opts) => {
                let data_only = opts.data_on_stdout();
                if !data_only {
                    display_logo(opts.common.quiet, opts.common.no_color, true, false);
                }
                let code = registry::networks::run(opts);
                if !data_only {
                    print_elapsed(start, opts.common.no_color);
                }
                code
            }
        },
    };
    std::process::exit(code);
}

/// The analyst computer's local time, as `Start time` and `End time` show it.
fn local_time() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn print_elapsed(start: Instant, no_color: bool) {
    p(Green.rdg(no_color), "End time: ", false);
    p(None, &local_time(), true);
    p(Green.rdg(no_color), "Elapsed time: ", false);
    p(None, &format!("{}\n", elapsed_text(start.elapsed())), true);
}

/// `HH:MM:SS`, rounded to the nearest second.
fn elapsed_text(duration: std::time::Duration) -> String {
    let total = (duration.as_millis() + 500) / 1000;
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}

fn display_logo(quiet: bool, no_color: bool, time: bool, help: bool) {
    // The logo ends with a newline, so printing it with one leaves a single blank line after it.
    if !quiet {
        p(Green.rdg(no_color), LOGO, true);
    }
    if help {
        p(None, &format!("Version: {VERSION}"), true);
    } else {
        p(Green.rdg(no_color), "Version: ", false);
        p(None, VERSION, true);
    }
    if time {
        p(Green.rdg(no_color), "Start time: ", false);
        p(None, &local_time(), true);
    }
    p(None, "", true);
}

#[cfg(test)]
mod tests {
    use super::elapsed_text;
    use std::time::Duration;

    #[test]
    fn elapsed_time_is_rounded_to_the_nearest_second() {
        for (millis, text) in [
            (0, "00:00:00"),
            (499, "00:00:00"),
            (500, "00:00:01"),
            (1_499, "00:00:01"),
            (59_500, "00:01:00"),
            (3_599_499, "00:59:59"),
            (3_599_500, "01:00:00"),
            (90_061_000, "25:01:01"),
        ] {
            assert_eq!(
                elapsed_text(Duration::from_millis(millis)),
                text,
                "{millis}"
            );
        }
    }
}
