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
mod cli;
mod color;
mod defender_quarantine;
mod hive;
mod i30;
mod jump_lists;
mod lnk;
mod logfile;
mod mft;
mod mft_csv;
mod mft_inspect;
mod output_schema;
mod persistence_hunter;
mod prefetch;
mod progress;
mod registry;
mod security_descriptor;
mod shimcache;
mod srum;
mod super_timeline;
mod usn_journal;
mod util;

use chrono::Local;
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands, EseCommands, FileCommands, NtfsCommands, RegistryCommands, VERSION};
use color::JunteiColor::Green;
use std::env;
use std::time::Instant;
use util::p;

const LOGO: &str = include_str!("../art/logo.txt");

fn main() {
    let args: Vec<String> = env::args().collect();
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
        Commands::ParseNtfs(source) => match &source.cmd {
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
            RegistryCommands::Recentdocs(opts) => {
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
            RegistryCommands::Shellbags(opts) => {
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
            RegistryCommands::Userassist(opts) => {
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
            RegistryCommands::Opensavemru(opts) => {
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
            RegistryCommands::Featureusage(opts) => {
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
            RegistryCommands::Appcompat(opts) => {
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

fn print_elapsed(start: Instant, no_color: bool) {
    let duration = start.elapsed();
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    let millis = duration.subsec_millis();
    p(Green.rdg(no_color), "Elapsed time: ", false);
    p(
        None,
        &format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}\n"),
        true,
    );
}

fn display_logo(quiet: bool, no_color: bool, time: bool, help: bool) {
    if !quiet {
        p(Green.rdg(no_color), LOGO, true);
        p(None, "", true);
    }
    if time {
        let msg = Local::now().format("%Y/%m/%d %H:%M").to_string();
        p(Green.rdg(no_color), "Start time: ", false);
        p(None, &msg, true);
    }
    if help {
        p(None, &format!("Version: {VERSION}"), true);
    } else {
        p(Green.rdg(no_color), "Version: ", false);
        p(None, VERSION, true);
    }
    p(None, "", true);
}
