//! Independent registry files with controlled persistence configuration, parsed by the real
//! native hive reader. These test registry traversal and semantics, not mocks of the reader.
use super::*;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
struct Node {
    children: BTreeMap<String, Node>,
    values: BTreeMap<String, (u32, Vec<u8>)>,
}
impl Node {
    fn put(&mut self, path: &str, name: &str, kind: u32, data: Vec<u8>) {
        let mut node = self;
        for part in path.split('\\').filter(|s| !s.is_empty()) {
            node = node.children.entry(part.to_string()).or_default();
        }
        node.values.insert(name.to_string(), (kind, data));
    }
    fn text(&mut self, path: &str, name: &str, data: &str) {
        self.put(path, name, 1, utf16(data));
    }
    fn number(&mut self, path: &str, name: &str, n: u32) {
        self.put(path, name, 4, n.to_le_bytes().to_vec());
    }
    fn multi(&mut self, path: &str, name: &str, values: &[&str]) {
        self.put(path, name, 7, utf16(&values.join("\0")));
    }
}
fn utf16(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect()
}
struct Builder {
    bin: Vec<u8>,
}
impl Builder {
    fn new() -> Self {
        Self { bin: vec![0; 32] }
    }
    fn cell(&mut self, data: &[u8]) -> u32 {
        let p = self.bin.len();
        let size = (data.len() + 4).next_multiple_of(8);
        self.bin.extend((-(size as i32)).to_le_bytes());
        self.bin.extend(data);
        self.bin.resize(p + size, 0);
        p as u32
    }
    fn key(&mut self, name: &str, node: &Node, root: bool) -> u32 {
        let children: Vec<_> = node
            .children
            .iter()
            .map(|(n, c)| self.key(n, c, false))
            .collect();
        let mut values = vec![];
        for (n, (kind, raw)) in &node.values {
            let (len, ptr) = if raw.len() <= 4 {
                let mut a = [0; 4];
                a[..raw.len()].copy_from_slice(raw);
                (0x80000000 | raw.len() as u32, u32::from_le_bytes(a))
            } else {
                (raw.len() as u32, self.cell(raw))
            };
            let mut v = vec![0; 0x14];
            v[..2].copy_from_slice(b"vk");
            let name_bytes: Vec<_> = if n.is_ascii() {
                n.as_bytes().to_vec()
            } else {
                n.encode_utf16().flat_map(u16::to_le_bytes).collect()
            };
            v[2..4].copy_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            v[4..8].copy_from_slice(&len.to_le_bytes());
            v[8..12].copy_from_slice(&ptr.to_le_bytes());
            v[12..16].copy_from_slice(&kind.to_le_bytes());
            v[16..18].copy_from_slice(&u16::from(n.is_ascii()).to_le_bytes());
            v.extend(name_bytes);
            values.push(self.cell(&v));
        }
        let sub = if children.is_empty() {
            u32::MAX
        } else {
            let mut list = b"li".to_vec();
            list.extend((children.len() as u16).to_le_bytes());
            for p in &children {
                list.extend(p.to_le_bytes());
            }
            self.cell(&list)
        };
        let val = if values.is_empty() {
            u32::MAX
        } else {
            self.cell(
                &values
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .collect::<Vec<_>>(),
            )
        };
        let mut k = vec![0; 0x4c];
        k[..2].copy_from_slice(b"nk");
        k[2..4].copy_from_slice(&(if root { 0x2c_u16 } else { 0x20 }).to_le_bytes());
        k[4..12].copy_from_slice(&132888278320000000u64.to_le_bytes());
        for (p, v) in [
            (0x10, u32::MAX),
            (0x14, children.len() as u32),
            (0x1c, sub),
            (0x20, u32::MAX),
            (0x24, values.len() as u32),
            (0x28, val),
            (0x2c, u32::MAX),
            (0x30, u32::MAX),
        ] {
            k[p..p + 4].copy_from_slice(&v.to_le_bytes());
        }
        k[0x48..0x4a].copy_from_slice(&(name.len() as u16).to_le_bytes());
        k.extend(name.as_bytes());
        self.cell(&k)
    }
    fn hive(mut self, node: &Node, name: &str) -> Vec<u8> {
        let root = self.key("ROOT", node, true);
        let size = (self.bin.len() + 8).next_multiple_of(4096);
        let remain = size - self.bin.len();
        self.bin.extend((remain as i32).to_le_bytes());
        self.bin.resize(size, 0);
        self.bin[..4].copy_from_slice(b"hbin");
        self.bin[8..12].copy_from_slice(&(size as u32).to_le_bytes());
        let mut h = vec![0; 4096];
        h[..4].copy_from_slice(b"regf");
        for (p, v) in [
            (4, 1u32),
            (8, 1),
            (20, 1),
            (24, 5),
            (32, 1),
            (36, root),
            (40, size as u32),
        ] {
            h[p..p + 4].copy_from_slice(&v.to_le_bytes());
        }
        for (i, b) in utf16(name).into_iter().take(64).enumerate() {
            h[0x30 + i] = b;
        }
        let sum = (0..508).step_by(4).fold(0u32, |a, i| {
            a ^ u32::from_le_bytes(h[i..i + 4].try_into().unwrap())
        });
        h[508..512].copy_from_slice(&sum.to_le_bytes());
        h.extend(self.bin);
        h
    }
}
fn save(root: &Path, kind: &str, tree: &Node) {
    let path = if kind == "NTUSER.DAT" {
        root.join("Users/fixture/NTUSER.DAT")
    } else if kind == "USRCLASS.DAT" {
        root.join("Users/fixture/AppData/Local/Microsoft/Windows/UsrClass.dat")
    } else {
        root.join("Windows/System32/config").join(kind)
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, Builder::new().hive(tree, kind)).unwrap();
}

#[test]
fn every_declarative_check_reads_real_native_hive() {
    for check in catalogue()
        .iter()
        .filter(|c| c.implementation == "declarative" || c.id.starts_with("winlogon_"))
    {
        let t = &check.targets[0];
        let dir = tempfile::tempdir().unwrap();
        let mut tree = Node::default();
        let (kind, path) = if t.scope == "HKU" {
            ("NTUSER.DAT", t.path.clone())
        } else {
            let (k, p) = t.path.split_once('\\').unwrap();
            (k, p.replace("{controlset}", "ControlSet001"))
        };
        let path = if t.recurse {
            join(&[&path, "fixture-entry"])
        } else {
            path
        };
        let value_name = if t.values == "*" {
            "FixtureValue"
        } else if t.values == "(Default)" {
            ""
        } else {
            &t.values
        };
        tree.text(&path, value_name, r"C:\fixture.exe");
        save(dir.path(), kind, &tree);
        let ctx = Context::open(dir.path(), true).unwrap();
        let mut run = CheckRun::new(&ctx, check);
        run.declarative();
        assert!(
            !run.findings.is_empty(),
            "{} did not read its declared native hive target {}",
            check.id,
            path
        );
        assert!(run.findings.iter().any(|f| f.value == r"C:\fixture.exe"));
        assert!(
            run.findings
                .iter()
                .all(|f| !f.source_file.is_empty() && !f.key_last_write.is_empty())
        );
    }
}

#[test]
fn custom_checks_find_controlled_native_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let mut sw = Node::default();
    let mut sys = Node::default();
    let mut user = Node::default();
    let mut classes = Node::default();
    let mut sam = Node::default();
    let id = "{11111111-1111-1111-1111-111111111111}";
    let server = join(&[r"Classes\CLSID", id, "InprocServer32"]);
    sw.text(&server, "", r"C:\fixture.dll");
    for p in [
        r"Microsoft\Active Setup\Installed Components\fixture",
        r"Wow6432Node\Microsoft\Active Setup\Installed Components\fixture",
    ] {
        sw.text(p, "StubPath", r"C:\fixture.exe");
    }
    sw.text(
        r"Microsoft\Windows\CurrentVersion\App Paths\fixture.exe",
        "",
        r"C:\fixture.exe",
    );
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion\Windows",
        "AppInit_DLLs",
        "bad.dll other.dll",
    );
    sw.number(
        r"Microsoft\Windows NT\CurrentVersion\Windows",
        "LoadAppInit_DLLs",
        0,
    );
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\fixture",
        "StartExe",
        r"C:\fixture.exe",
    );
    user.text(
        r"Software\Microsoft\Windows NT\CurrentVersion\Accessibility",
        "Configuration",
        "fixture",
    );
    sw.text(&join(&[r"Microsoft\AMSI\Providers", id]), "", "");
    sw.text(
        &join(&[
            r"Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects",
            id,
        ]),
        "",
        "",
    );
    sw.text(
        r"Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\fixture",
        "",
        id,
    );
    sw.text(
        r"Microsoft\Windows\CurrentVersion\Explorer\AppKey\99",
        "ShellExecute",
        r"C:\fixture.exe",
    );
    sw.text(
        r"Classes\Directory\shellex\ContextMenuHandlers\fixture",
        "",
        id,
    );
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController\fixture",
        "Binary",
        "bad.dll",
    );
    let ifeo =
        r"Microsoft\Windows NT\CurrentVersion\Image File Execution Options\fixture.exe\nested";
    for n in ["Debugger", "VerifierDlls"] {
        sw.text(ifeo, n, r"C:\fixture.dll");
    }
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion\SilentProcessExit\fixture.exe",
        "MonitorProcess",
        r"C:\fixture.exe",
    );
    user.text(r"Control Panel\Desktop", "SCRNSAVE.EXE", r"C:\fixture.scr");
    sys.text(
        r"ControlSet001\Control\ContentIndex\Language\fixture",
        "DLLOverridePath",
        r"C:\fixture.dll",
    );
    sys.text(
        r"ControlSet001\Control\Session Manager\KnownDLLs",
        "fixture",
        "fixture.dll",
    );
    sys.multi(
        r"ControlSet001\Control\Session Manager",
        "ExcludeFromKnownDlls",
        &["fixture.dll"],
    );
    sys.number(
        r"ControlSet001\Control\Session Manager",
        "SafeDllSearchMode",
        0,
    );
    for n in [
        "DOTNET_STARTUP_HOOKS",
        "COR_PROFILER_PATH",
        "CORECLR_PROFILER_PATH",
    ] {
        sys.text(
            r"ControlSet001\Control\Session Manager\Environment",
            n,
            r"C:\fixture.dll",
        );
    }
    sys.number(r"ControlSet001\Control\Lsa", "DsrmAdminLogonBehavior", 2);
    sys.multi(
        r"ControlSet001\Control\Lsa",
        "Notification Packages",
        &["fixture"],
    );
    let svc = r"ControlSet001\Services\fixture";
    sys.text(svc, "ImagePath", r"C:\fixture.exe");
    sys.text(svc, "FailureCommand", r"C:\recovery.cmd");
    sys.text(&join(&[svc, "Parameters"]), "ServiceDll", r"C:\fixture.dll");
    sys.text(
        &join(&[svc, "NetworkProvider"]),
        "ProviderPath",
        r"C:\fixture.dll",
    );
    sw.text(
        r"Microsoft\Terminal Server Client\Default\AddIns\fixture",
        "Name",
        "fixture.dll",
    );
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls",
        "trigger.dll",
        "fixture.dll",
    );
    sw.number(
        r"Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules",
        r"C:\fixture.dll",
        0,
    );
    sw.number(
        r"Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\fixture.exe",
        "fixture.sdb",
        0,
    );
    sw.text(
        r"Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI",
        "fixture",
        "bad.dll",
    );
    sw.text(
        r"Microsoft\Office\16.0\Word",
        "WwlibtDll",
        r"C:\fixture.dll",
    );
    sw.text(
        r"Microsoft\Office\16.0\Word\Addins\fixture",
        "Manifest",
        r"C:\fixture.vsto",
    );
    user.text(
        r"Software\Microsoft\Office\16.0\Outlook\WebView\Inbox",
        "URL",
        "https://fixture.invalid/",
    );
    sw.text(
        r"Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32",
        "",
        r"C:\fixture.dll",
    );
    sw.text(r"Classes\.txt\shell\open\command", "", r"C:\fixture.exe %1");
    sw.text(r"Classes\http\shell\open\command", "", r"C:\fixture.exe %1");
    sw.text(&join(&[r"Classes\CLSID", id, "TreatAs"]), "", id);
    classes.text(
        r"TypeLib\{22222222-2222-2222-2222-222222222222}\1.0\0\win32",
        "",
        r"C:\fixture.dll\1",
    );
    sw.number(
        r"Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList",
        "fixture",
        0,
    );
    sw.text(
        r"Microsoft\Windows\CurrentVersion\RunOnceEx\001\nested",
        "Command",
        r"C:\fixture.exe",
    );
    sw.text(r"Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx\001","Command",r"C:\fixture.exe");
    let mut f = vec![0; 0x50];
    f[0x30..0x34].copy_from_slice(&500u32.to_le_bytes());
    sam.put(r"SAM\Domains\Account\Users\000003E8", "F", 3, f);
    for (kind, tree) in [
        ("SOFTWARE", &sw),
        ("SYSTEM", &sys),
        ("NTUSER.DAT", &user),
        ("USRCLASS.DAT", &classes),
        ("SAM", &sam),
    ] {
        save(dir.path(), kind, tree);
    }
    let ctx = Context::open(dir.path(), true).unwrap();
    let results = run(&ctx);
    let ids: BTreeSet<_> = results
        .findings
        .iter()
        .map(|f| f.check_id.as_str())
        .collect();
    for check in catalogue().iter().filter(|c| {
        c.implementation == "custom"
            && !FILE_CHECKS.contains(&c.id.as_str())
            && !c.id.starts_with("winlogon_")
            && c.id != "admin_group_membership"
    }) {
        assert!(
            ids.contains(check.id.as_str()),
            "Custom registry check {} did not find its native fixture",
            check.id
        );
    }
    let appinit = results
        .findings
        .iter()
        .filter(|f| f.check_id == "appinit_dlls")
        .collect::<Vec<_>>();
    assert_eq!(appinit.len(), 2);
    assert!(appinit.iter().all(|f| f.value.contains("INACTIVE")));
    let lsa = results
        .findings
        .iter()
        .find(|f| f.check_id == "lsa_password_filter")
        .unwrap();
    assert_eq!(lsa.resolve_target, r"Windows\System32\fixture.dll");
}

#[test]
fn missing_hives_are_not_reported_as_complete_clean_checks() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = run(&ctx);
    assert!(scan.findings.is_empty());
    assert!(scan.coverage.iter().all(|c| c.status == "not_collected"));
}

fn binary_sid(parts: &[u32]) -> Vec<u8> {
    let mut b = vec![1, parts.len() as u8, 0, 0, 0, 0, 0, 5];
    for n in parts {
        b.extend(n.to_le_bytes());
    }
    b
}
#[test]
fn administrator_membership_ignores_valid_groups_and_foreign_accounts() {
    let dir = tempfile::tempdir().unwrap();
    let mut sam = Node::default();
    let mut sw = Node::default();
    sam.put(r"SAM\Domains\Account", "V", 3, binary_sid(&[21, 1, 2, 3]));
    let name = utf16("hidden-admin");
    let mut v = vec![0; 0xcc];
    v[0x10..0x14].copy_from_slice(&(name.len() as u32).to_le_bytes());
    v.extend(name);
    sam.put(r"SAM\Domains\Account\Users\000003E8", "V", 3, v);
    sam.number(r"SAM\Domains\Account\Groups\000003EB", "fixture", 0);
    let mut members = vec![];
    for parts in [
        [21, 1, 2, 3, 1000],
        [21, 1, 2, 3, 1001],
        [21, 1, 2, 3, 1003],
        [21, 9, 9, 9, 1002],
    ] {
        members.extend(binary_sid(&parts));
    }
    let mut c = vec![0; 0x34];
    c[0x2c..0x30].copy_from_slice(&(members.len() as u32).to_le_bytes());
    c.extend(members);
    sam.put(r"SAM\Domains\Builtin\Aliases\00000220", "C", 3, c);
    sw.number(
        r"Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList",
        "hidden-admin",
        0,
    );
    save(dir.path(), "SAM", &sam);
    save(dir.path(), "SOFTWARE", &sw);
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = run(&ctx);
    let findings: Vec<_> = scan
        .findings
        .iter()
        .filter(|f| f.check_id == "admin_group_membership")
        .collect();
    assert_eq!(findings.len(), 2);
    assert!(findings.iter().any(|f| f.value.contains("hidden-admin")));
    assert!(
        findings
            .iter()
            .any(|f| f.value.contains("1001") && f.value.contains("no account or group"))
    );
    assert!(
        findings
            .iter()
            .all(|f| !f.value.contains("1003") && !f.value.contains("1002"))
    );
}

fn save_user_classes(root: &Path, user: &str, tree: &Node) {
    let p = root
        .join("Users")
        .join(user)
        .join("AppData/Local/Microsoft/Windows/UsrClass.dat");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, Builder::new().hive(tree, "UsrClass.dat")).unwrap();
}
#[test]
fn treat_as_resolves_same_users_server_before_machine_and_never_other_users() {
    let dir = tempfile::tempdir().unwrap();
    let mut sw = Node::default();
    let mut alice = Node::default();
    let mut bob = Node::default();
    let source = "{AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA}";
    let target = "{BBBBBBBB-BBBB-BBBB-BBBB-BBBBBBBBBBBB}";
    alice.text(&join(&["CLSID", source, "TreatAs"]), "", target);
    bob.text(
        &join(&["CLSID", target, "InprocServer32"]),
        "",
        r"C:\BobPrivate.dll",
    );
    save_user_classes(dir.path(), "Alice", &alice);
    save_user_classes(dir.path(), "Bob", &bob);
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = run(&ctx);
    let alice_finding = scan
        .findings
        .iter()
        .find(|f| {
            f.check_id == "com_treat_as" && f.path.contains("Alice") && f.path.ends_with("TreatAs")
        })
        .unwrap();
    assert_eq!(alice_finding.resolve_target, "");
    sw.text(
        &join(&[r"Classes\CLSID", target, "InprocServer32"]),
        "",
        r"C:\Machine.dll",
    );
    alice.text(
        &join(&["CLSID", target, "InprocServer32"]),
        "",
        r"C:\AlicePrivate.dll",
    );
    save(dir.path(), "SOFTWARE", &sw);
    save_user_classes(dir.path(), "Alice", &alice);
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = run(&ctx);
    let alice_finding = scan
        .findings
        .iter()
        .find(|f| {
            f.check_id == "com_treat_as" && f.path.contains("Alice") && f.path.ends_with("TreatAs")
        })
        .unwrap();
    assert_eq!(alice_finding.resolve_target, r"C:\AlicePrivate.dll");
}

#[test]
fn damaged_registry_values_produce_partial_coverage() {
    let dir = tempfile::tempdir().unwrap();
    let mut sw = Node::default();
    sw.put(
        r"Microsoft\Windows\CurrentVersion\App Paths\fixture.exe",
        "",
        4,
        vec![1, 2, 3],
    );
    save(dir.path(), "SOFTWARE", &sw);
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = run(&ctx);
    let coverage = scan
        .coverage
        .iter()
        .find(|c| c.check_id == "app_paths")
        .unwrap();
    assert_eq!(coverage.status, "partial");
    assert!(coverage.detail.contains("3 bytes"));
}

#[test]
fn winlogon_comma_separated_commands_resolve_separately() {
    let dir = tempfile::tempdir().unwrap();
    let mut sw = Node::default();
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion\Winlogon",
        "Userinit",
        r"C:\Windows\system32\userinit.exe, C:\bad.exe,,",
    );
    save(dir.path(), "SOFTWARE", &sw);
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = run(&ctx);
    let values: Vec<_> = scan
        .findings
        .iter()
        .filter(|f| f.check_id == "winlogon_userinit")
        .map(|f| f.value.as_str())
        .collect();
    assert_eq!(
        values,
        vec![r"C:\Windows\system32\userinit.exe", r"C:\bad.exe"]
    );
}

#[test]
fn selected_checks_do_not_execute_other_registry_checks() {
    let dir = tempfile::tempdir().unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    let check = catalogue()
        .iter()
        .find(|c| c.id == "run_keys")
        .unwrap()
        .clone();
    let scan = run_definitions(&ctx, &[check]);
    assert_eq!(scan.coverage.len(), 1);
    assert_eq!(scan.coverage[0].check_id, "run_keys");
    assert!(scan.findings.is_empty());
}

#[test]
fn cyclic_ifeo_key_is_reported_as_partial_without_duplicate_findings() {
    let dir = tempfile::tempdir().unwrap();
    let mut sw = Node::default();
    let base = r"Microsoft\Windows NT\CurrentVersion\Image File Execution Options";
    let image = join(&[base, "fixture.exe"]);
    sw.text(&image, "Debugger", r"C:\fixture.exe");
    sw.text(&join(&[&image, "unused-child"]), "", "");
    let mut bytes = Builder::new().hive(&sw, "SOFTWARE");
    let (hive, _) = registry::Hive::open(bytes.clone(), &[]).unwrap();
    let offset = hive.key(&image).unwrap().offset();
    let key_start = 4096 + offset as usize + 4;
    let list = u32::from_le_bytes(
        bytes[key_start + 0x1c..key_start + 0x20]
            .try_into()
            .unwrap(),
    );
    // Make fixture.exe's sole child point back to its IFEO parent. This is a
    // two-key cycle; immediate self-links are already rejected by the hive reader.
    let entry = 4096 + list as usize + 8;
    let parent_offset = hive.key(base).unwrap().offset();
    bytes[entry..entry + 4].copy_from_slice(&parent_offset.to_le_bytes());
    let p = dir.path().join("Windows/System32/config/SOFTWARE");
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, bytes).unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    let check = catalogue()
        .iter()
        .find(|c| c.id == "ifeo_debugger")
        .unwrap()
        .clone();
    let scan = run_definitions(&ctx, &[check]);
    assert_eq!(scan.findings.len(), 1);
    assert_eq!(scan.coverage[0].status, "partial");
    assert!(scan.coverage[0].detail.contains("cycle"));
}

#[test]
fn shared_service_images_cannot_date_a_different_services_installation() {
    use super::super::TimeEvidence;
    let dir = tempfile::tempdir().unwrap();
    let mut sys = Node::default();
    sys.number("Select", "Current", 1);
    sys.text(
        r"ControlSet001\Services\FirstService",
        "ImagePath",
        r"%SystemRoot%\System32\svchost.exe -k netsvcs",
    );
    sys.text(
        r"ControlSet001\Services\LaterService",
        "ImagePath",
        r#""C:\Windows\System32\SVCHOST.EXE"   -k NETSVCS"#,
    );
    sys.text(
        r"ControlSet001\Services\UniqueService",
        "ImagePath",
        r"C:\Unique\service.exe --standalone",
    );
    sys.text(
        r"ControlSet002\Services\InactiveService",
        "ImagePath",
        r"C:\Unique\service.exe --standalone",
    );
    save(dir.path(), "SYSTEM", &sys);
    let ctx = Context::open(dir.path(), true).unwrap();
    let check = catalogue()
        .iter()
        .find(|c| c.id == "windows_service_image_path")
        .unwrap()
        .clone();
    let scan = run_definitions(&ctx, &[check]);
    assert_eq!(scan.findings.len(), 3);
    for finding in scan.findings {
        let image_descriptors = finding
            .time_evidence
            .iter()
            .filter(
                |t| matches!(t,TimeEvidence::EventLog{match_field,..} if match_field=="ImagePath"),
            )
            .count();
        if finding.path.contains("UniqueService") {
            assert_eq!(image_descriptors, 1);
        } else {
            assert_eq!(image_descriptors, 0);
        }
        assert!(finding.time_evidence.iter().any(
            |t| matches!(t,TimeEvidence::EventLog{match_field,..} if match_field=="ServiceName")
        ));
    }
}

#[test]
fn unicode_registry_content_cannot_abort_persistence_or_timeline() {
    let dir = tempfile::tempdir().unwrap();
    let mut classes = Node::default();
    classes.text(r"TypeLib\{fixture}\1.0\0\win32", "", "K%APPDATA%\\evil.dll");
    save(dir.path(), "USRCLASS.DAT", &classes);
    let mut sw = Node::default();
    sw.text(
        r"Microsoft\Windows\CurrentVersion\Run",
        "K\\CurrentControlSet\\x",
        r"C:\agent.exe",
    );
    sw.text(
        r"Microsoft\Windows\CurrentVersion\Run",
        "K\\ControlSet001\\x",
        r"C:\agent2.exe",
    );
    save(dir.path(), "SOFTWARE", &sw);
    let ctx = Context::open(dir.path(), true).unwrap();
    let mut scan = run(&ctx);
    assert!(scan.findings.iter().any(|f| f.check_id == "typelib_hijack"
        && f.resolve_target.contains(r"Users\fixture\AppData\Roaming")));
    assert!(scan.findings.iter().any(|f| f.path.contains('K')));
    crate::persistence_sniper::timeline::enrich(&ctx, &mut scan.findings, None);
}

#[test]
fn canonical_hives_cannot_be_replaced_by_user_writable_decoys_or_shadow_copies() {
    let dir = tempfile::tempdir().unwrap();
    let mut real = Node::default();
    real.text(
        r"Microsoft\Windows\CurrentVersion\Run",
        "Real",
        r"C:\real.exe",
    );
    save(dir.path(), "SOFTWARE", &real);
    let mut user = Node::default();
    user.text(
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        "Real",
        r"C:\user.exe",
    );
    save(dir.path(), "NTUSER.DAT", &user);
    let mut decoy = Node::default();
    decoy.text(
        r"Microsoft\Windows\CurrentVersion\Run",
        "Decoy",
        r"C:\decoy.exe",
    );
    for path in [
        "Users/Public/Documents/C/Windows/System32/config/SOFTWARE",
        "Users/fixture/AppData/Local/Temp/NTUSER.DAT",
        "uploads/auto/%5C%5C%3F%5CGLOBALROOT%5CDevice%5CHarddiskVolumeShadowCopy1/Users/fixture/NTUSER.DAT",
    ] {
        let path = dir.path().join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let kind = if path.ends_with("SOFTWARE") {
            "SOFTWARE"
        } else {
            "NTUSER.DAT"
        };
        std::fs::write(path, Builder::new().hive(&decoy, kind)).unwrap();
    }
    let ctx = Context::open(dir.path(), true).unwrap();
    assert_eq!(ctx.hives.len(), 2);
    let scan = run(&ctx);
    assert!(scan.findings.iter().any(|f| f.value == r"C:\real.exe"));
    assert!(scan.findings.iter().any(|f| f.value == r"C:\user.exe"));
    assert!(!scan.findings.iter().any(|f| f.value == r"C:\decoy.exe"));
}

#[test]
fn canonical_hive_is_preferred_and_bad_copy_can_fall_back_to_explicit_export() {
    let dir = tempfile::tempdir().unwrap();
    let mut real = Node::default();
    real.text(
        r"Microsoft\Windows\CurrentVersion\Run",
        "Real",
        r"C:\real.exe",
    );
    save(dir.path(), "SOFTWARE", &real);
    std::fs::write(dir.path().join("SOFTWARE"), b"bad exported copy").unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    assert!(
        ctx.machine_hive("SOFTWARE")
            .unwrap()
            .path
            .ends_with("Windows/System32/config/SOFTWARE")
    );
    std::fs::write(
        dir.path().join("Windows/System32/config/SOFTWARE"),
        b"bad canonical copy",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("SOFTWARE"),
        Builder::new().hive(&real, "SOFTWARE"),
    )
    .unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    assert_eq!(
        ctx.machine_hive("SOFTWARE").unwrap().path,
        dir.path().join("SOFTWARE").canonicalize().unwrap()
    );
    assert!(ctx.issues.iter().any(|i| i.contains("Cannot parse hive")));
}

#[test]
fn a_volume_named_windows_retains_machine_hives_and_user_hives() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("windows");
    let mut sw = Node::default();
    sw.text(
        r"Microsoft\Windows\CurrentVersion\Run",
        "Probe",
        r"C:\probe.exe",
    );
    save(&root, "SOFTWARE", &sw);
    save(&root, "NTUSER.DAT", &Node::default());
    let ctx = Context::open(&root, true).unwrap();
    assert!(ctx.machine_hive("SOFTWARE").is_some());
    assert_eq!(ctx.hives.len(), 2);
    assert!(
        run(&ctx)
            .findings
            .iter()
            .any(|f| f.value == r"C:\probe.exe")
    );
    assert_eq!(
        crate::auto_parse::evidence_roots(&root, 10).unwrap(),
        vec![root]
    );
}

#[test]
fn memory_budget_keeps_machine_hives_and_file_checks_and_reports_skips() {
    let dir = tempfile::tempdir().unwrap();
    save(dir.path(), "SYSTEM", &Node::default());
    save(dir.path(), "SOFTWARE", &Node::default());
    save(dir.path(), "NTUSER.DAT", &Node::default());
    let profile = dir
        .path()
        .join("Users/fixture/Documents/PowerShell/profile.ps1");
    std::fs::create_dir_all(profile.parent().unwrap()).unwrap();
    std::fs::write(&profile, "Write-Output fixture").unwrap();
    let ctx = Context::open_with_budget(dir.path(), true, 16_384).unwrap();
    assert!(ctx.machine_hive("SYSTEM").is_some());
    assert!(ctx.machine_hive("SOFTWARE").is_some());
    assert!(!ctx.hives.iter().any(|h| h.kind == "NTUSER.DAT"));
    assert!(
        ctx.issues
            .iter()
            .any(|i| i.contains("Skipped NTUSER.DAT") && i.contains("memory budget"))
    );
    let scan = crate::persistence_sniper::file_checks::run(&ctx, catalogue());
    assert!(
        scan.findings
            .iter()
            .any(|f| f.check_id == "powershell_profiles")
    );
    let log = dir.path().join("Windows/System32/config/SYSTEM.LOG1");
    std::fs::write(log, vec![0; 16_384]).unwrap();
    let ctx = Context::open_with_budget(dir.path(), false, 16_384).unwrap();
    assert!(ctx.machine_hive("SYSTEM").is_some());
    assert!(
        ctx.issues
            .iter()
            .any(|i| i.contains("Skipped transaction log"))
    );
}

#[test]
fn original_host_drive_is_mapped_to_the_captured_system_volume() {
    use crate::persistence_sniper::{context::Resolution, resolver};
    let dir = tempfile::tempdir().unwrap();
    let captured = dir.path().join("E");
    let mut sw = Node::default();
    sw.text(
        r"Microsoft\Windows NT\CurrentVersion",
        "SystemRoot",
        r"C:\Windows",
    );
    save(&captured, "SOFTWARE", &sw);
    std::fs::write(captured.join("Windows/System32/agent.exe"), b"test").unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    assert_eq!(ctx.system_drive(), 'E');
    assert_eq!(ctx.original_system_drive(), 'C');
    for value in [
        r"C:\Windows\System32\agent.exe",
        r"%SystemRoot%\System32\agent.exe",
        r"System32\agent.exe",
    ] {
        let mut finding = Finding {
            value: value.into(),
            ..Default::default()
        };
        resolver::enrich(&ctx, &mut finding);
        assert_eq!(
            finding.resolution_status, "collected",
            "{value}: {}",
            finding.resolution_status
        );
        assert_eq!(finding.is_in_os_directory, Some(true));
        assert!(!finding.sha256.is_empty());
    }
    assert!(matches!(
        ctx.resolve(r"D:\Windows\System32\agent.exe"),
        Resolution::NotCollected
    ));
    assert!(matches!(
        ctx.resolve(r"E:\Windows\System32\agent.exe"),
        Resolution::NotCollected
    ));
}

#[test]
fn startup_redirects_share_bounded_environment_expansion_and_keep_lossy_registry_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut user = Node::default();
    let key = r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";
    let mut bytes = utf16(r"%TEMP%\Startup");
    bytes.push(0x41); // Windows accepts a trailing odd byte on this text value.
    user.put(key, "Startup", 2, bytes);
    user.text("Environment", "TEMP", r"%LOCALAPPDATA%\Temp");
    save(dir.path(), "NTUSER.DAT", &user);
    let payload = dir
        .path()
        .join("Users/fixture/AppData/Local/Temp/Startup/agent.exe");
    std::fs::create_dir_all(payload.parent().unwrap()).unwrap();
    std::fs::write(&payload, b"test").unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = crate::persistence_sniper::file_checks::run(&ctx, catalogue());
    assert!(
        scan.findings
            .iter()
            .any(|f| f.check_id == "startup_folder" && f.value.ends_with("agent.exe"))
    );
    assert!(scan.coverage.iter().any(|c| c.check_id == "startup_folder"
        && c.status == "partial"
        && c.detail.contains("UTF-16")));
}

#[test]
fn captured_host_names_include_dns_pending_and_client_identities() {
    let dir = tempfile::tempdir().unwrap();
    let mut system = Node::default();
    system.text(
        r"ControlSet001\Control\ComputerName\ComputerName",
        "ComputerName",
        "NETBIOS-LIMITED",
    );
    system.text(
        r"ControlSet001\Control\ComputerName\ActiveComputerName",
        "ComputerName",
        "CURRENT",
    );
    system.text(
        r"ControlSet001\Services\Tcpip\Parameters",
        "Hostname",
        "long-dns-host-name",
    );
    system.text(
        r"ControlSet001\Services\Tcpip\Parameters",
        "NV Hostname",
        "PENDING",
    );
    save(dir.path(), "SYSTEM", &system);
    std::fs::write(
        dir.path().join("client_info.json"),
        br#"{"hostname":"CLIENT","fqdn":"client.example.test"}"#,
    )
    .unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    let identities = ctx.host_identities();
    for name in [
        "NETBIOS-LIMITED",
        "CURRENT",
        "long-dns-host-name",
        "PENDING",
        "CLIENT",
        "client.example.test",
    ] {
        assert!(identities.iter().any(|(n, _)| n == name), "{name}");
    }
}

#[test]
fn service_account_default_startup_is_not_misreported_as_a_redirect() {
    for with_software in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let mut sw = Node::default();
        sw.text(
            r"Microsoft\Windows NT\CurrentVersion\ProfileList\S-1-5-19",
            "ProfileImagePath",
            r"C:\Windows\ServiceProfiles\LocalService",
        );
        if with_software {
            save(dir.path(), "SOFTWARE", &sw);
        }
        let mut user = Node::default();
        user.text(
            r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders",
            "Startup",
            r"%USERPROFILE%\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup",
        );
        let path = dir
            .path()
            .join("Windows/ServiceProfiles/LocalService/NTUSER.DAT");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, Builder::new().hive(&user, "NTUSER.DAT")).unwrap();
        let ctx = Context::open(dir.path(), true).unwrap();
        let scan = crate::persistence_sniper::file_checks::run(&ctx, catalogue());
        assert!(
            !scan
                .findings
                .iter()
                .any(|f| f.check_id == "shell_folders_startup")
        );
    }
}

#[test]
fn exported_hives_next_to_a_system32_export_are_not_rebased_as_windows_contents() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("System32")).unwrap();
    std::fs::write(dir.path().join("System32/agent.exe"), b"test").unwrap();
    let bytes = Builder::new().hive(&Node::default(), "SOFTWARE");
    std::fs::write(dir.path().join("SOFTWARE"), bytes).unwrap();
    let ctx = Context::open(dir.path(), true).unwrap();
    assert!(ctx.machine_hive("SOFTWARE").is_some());
}

#[test]
fn startup_redirection_to_a_different_volume_is_preserved_and_scoped() {
    let dir = tempfile::tempdir().unwrap();
    let mut user = Node::default();
    user.text(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders",
        "Startup",
        r"D:\Users\fixture\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup",
    );
    save(&dir.path().join("C"), "NTUSER.DAT", &user);
    for (drive, name) in [("C", "wrong-volume.exe"), ("D", "right-volume.exe")] {
        let file = dir
            .path()
            .join(drive)
            .join("Users/fixture/AppData/Roaming/Microsoft/Windows/Start Menu/Programs/Startup")
            .join(name);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, b"test").unwrap();
    }
    let ctx = Context::open(dir.path(), true).unwrap();
    let scan = crate::persistence_sniper::file_checks::run(&ctx, catalogue());
    assert!(
        scan.findings
            .iter()
            .any(|f| f.check_id == "shell_folders_startup" && f.path.starts_with("HKU"))
    );
    assert!(
        scan.findings
            .iter()
            .any(|f| f.check_id == "startup_folder" && f.value.ends_with("right-volume.exe"))
    );
    assert!(
        !scan
            .findings
            .iter()
            .any(|f| f.check_id == "startup_folder" && f.value.ends_with("wrong-volume.exe"))
    );
}
