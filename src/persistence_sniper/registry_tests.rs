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
            v[2..4].copy_from_slice(&(n.len() as u16).to_le_bytes());
            v[4..8].copy_from_slice(&len.to_le_bytes());
            v[8..12].copy_from_slice(&ptr.to_le_bytes());
            v[12..16].copy_from_slice(&kind.to_le_bytes());
            v[16..18].copy_from_slice(&1u16.to_le_bytes());
            v.extend(n.as_bytes());
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
