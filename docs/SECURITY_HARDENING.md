# ZERO-DAY OS — Security Hardening Layer

Target: CardPuterZero (RP3A0/BCM2837, aarch64, 512MB RAM)
Version: 4.6.0
Goal: Zero-day exploit resistance via defence-in-depth

---

## Architecture

Security is layered across four planes:

1. **Kernel** — compile-time hardening flags in `kernel/zeroday-fragment.config`
2. **OS runtime** — sysctl policies, nftables firewall, AppArmor MAC, hidepid
3. **Storage** — dm-crypt LUKS2 encrypted vault + dm-verity rootfs integrity
4. **Attack surface reduction** — SUID audit, legacy protocol removal, SSH lockdown

---

## 1. Kernel Hardening (`kernel/zeroday-fragment.config`)

| Category           | Config Options                                                                 |
|--------------------|--------------------------------------------------------------------------------|
| KASLR              | `RANDOMIZE_BASE`, `RANDOMIZE_MEMORY`                                           |
| Stack/heap         | `SHADOW_CALL_STACK`, `INIT_ON_ALLOC_DEFAULT_ON`, `INIT_ON_FREE_DEFAULT_ON`, `ZERO_CALL_USED_REGS` |
| Spectre/Meltdown   | `HARDEN_BRANCH_PREDICTOR`, `HARDEN_EL2_VECTORS`, `UNMAP_KERNEL_AT_EL0`       |
| Slab               | `SLAB_FREELIST_RANDOM`, `SLAB_FREELIST_HARDENED`                               |
| Encrypted storage  | `DM_CRYPT` (LUKS2 AES-256-XTS)                                                 |
| Rootfs integrity   | `DM_VERITY`, `DM_VERITY_VERIFY_ROOTHASH_SIG`                                  |
| MAC                | `SECURITY_APPARMOR`, `SECURITY_YAMA`, `LSM=lockdown,yama,apparmor,bpf`        |
| Lockdown           | `SECURITY_LOCKDOWN_LSM`, `SECURITY_LOCKDOWN_LSM_EARLY` (integrity mode)       |
| IMA                | `IMA`, `IMA_APPRAISE`, `IMA_ARCH_POLICY`                                       |
| Audit              | `AUDIT`, `AUDITSYSCALL`                                                         |
| BPF                | `BPF_JIT_ALWAYS_ON`, unprivileged BPF disabled via sysctl                      |
| /dev hardening     | `STRICT_DEVMEM`, `IO_STRICT_DEVMEM`, `DEVKMEM=n`                              |
| RWX enforcement    | `STRICT_KERNEL_RWX`, `STRICT_MODULE_RWX`                                       |

---

## 2. OS Runtime Hardening (`pi-gen/stage3/12-security-hardening/`)

### sysctl (`/etc/sysctl.d/99-zeroday-hardening.conf`)

```
kernel.kptr_restrict = 2          # Prevent kernel pointer exposure
kernel.dmesg_restrict = 1         # Restrict dmesg to root
kernel.sysrq = 0                   # Disable magic SysRq
kernel.randomize_va_space = 2      # Full ASLR
kernel.unprivileged_bpf_disabled = 1
net.core.bpf_jit_harden = 2
kernel.yama.ptrace_scope = 1       # Restrict ptrace
kernel.kexec_load_disabled = 1     # Block runtime kernel replacement
vm.unprivileged_userfaultfd = 0    # Restrict userfaultfd
fs.protected_hardlinks/symlinks = 1
fs.protected_fifos/regular = 2
# Full network hardening: SYN cookies, no source routing, no ICMP redirects,
# reverse path filtering, no IPv6 RA, log martians
```

### nftables Firewall (`/etc/nftables.conf`)

Default-deny ingress policy. Allows:
- loopback
- established/related connections
- ICMP (rate-limited to 5 pps)
- mDNS (LAN only)
- Operator-defined TCP ports via the `allowed_tcp_in` set

Port additions at runtime:
```bash
nft add element inet zeroday allowed_tcp_in { 22 }   # enable SSH
nft delete element inet zeroday allowed_tcp_in { 22 } # revoke SSH
```

### AppArmor

Installed via stage3 `on_chroot`, enabled at boot. Use `aa-status` and
`aa-enforce /path/to/profile` to manage per-application MAC policies.

### hidepid=2

`/proc` mounted with `hidepid=2,gid=proc` — processes are only visible
to their owner. Prevents unprivileged enumeration of running processes.

---

## 3. Encrypted Storage (`scripts/system/zeroday-crypt`)

LUKS2 encrypted vault for loot and sensitive data.

```
Cipher:  AES-256-XTS (512-bit key)
KDF:     Argon2id (131072 KB memory, 4 parallel, 5000ms iter)
Hash:    SHA-512
```

### Workflow

```bash
# First-time setup (destructive — wipe /dev/mmcblk1p1 first)
zeroday-crypt format /dev/mmcblk1p1

# Daily use
zeroday-crypt open /dev/mmcblk1p1       # decrypt + mount to /opt/cardputer/vault
zeroday-crypt status
zeroday-crypt close                      # unmount + lock

# Key management
zeroday-crypt key-add /dev/mmcblk1p1    # add passphrase/keyfile slot
zeroday-crypt key-remove /dev/mmcblk1p1 # remove a slot

# Offline self-test
zeroday-crypt test
```

---

## 4. Rootfs Integrity (`scripts/system/zeroday-secureboot`)

dm-verity builds a Merkle hash tree over the read-only rootfs partition.
Any block-level tampering is detected at read time by the kernel.

### Workflow

```bash
# After image creation — compute and store root hash
zeroday-secureboot setup /dev/mmcblk0p2

# At runtime — verify hash chain
zeroday-secureboot verify /dev/mmcblk0p2

# Open verified read-only mapping
zeroday-secureboot open /dev/mmcblk0p2
mount -o ro /dev/mapper/zd-root-verity /mnt/verified-root

# Status
zeroday-secureboot status

# Offline test
zeroday-secureboot test
```

Stored at: `/boot/rootfs.verity.hash`, `/boot/rootfs.verity.salt`, `/boot/rootfs.verity.table`

---

## 5. Attack Surface Reduction

### SSH hardening (`/etc/ssh/sshd_config.d/99-zeroday-hardening.conf`)

- Password auth disabled (key-only)
- Only strong KEX: `curve25519-sha256`, `diffie-hellman-group16-sha512`
- Only strong ciphers: `chacha20-poly1305`, `aes256-gcm`
- MAC: `hmac-sha2-512-etm`, `hmac-sha2-256-etm`
- No X11/TCP/agent forwarding, no port tunnelling
- `LoginGraceTime 20`, `MaxAuthTries 3`, `PermitRootLogin prohibit-password`

### SUID/SGID audit (`/usr/local/bin/zeroday-suid-audit`)

```bash
zeroday-suid-audit              # list all SUID/SGID binaries + whitelist delta
zeroday-suid-audit --strip-sgid # strip SGID from non-essential binaries
```

### Disabled services

`rpcbind`, `rpc-statd`, `nfs-server`, `cups`, `bluetooth` (toggled via zd-power/zeroday-bt-mgr), `exim4`, `sendmail`, `postfix`, `telnetd`, `xinetd`

### Removed packages

`telnet`, `rsh-client`, `rsh-server`, `nis`, `yp-tools`, `tftp`

### Kernel cmdline hardening flags

```
apparmor=1 security=apparmor
init_on_alloc=1 init_on_free=1
slab_nomerge page_alloc.shuffle=1
pti=on vsyscall=none debugfs=off
oops=panic
lsm=lockdown,yama,apparmor,bpf
lockdown=integrity
```

---

## 6. Security Validation (`scripts/system/zeroday-security-check`)

```bash
zeroday-security-check              # interactive check, all categories
zeroday-security-check --json       # machine-readable JSON output
zeroday-security-check --mock       # simulate (no real hardware needed)
zeroday-security-check --verify     # exit 1 on any FAIL
zeroday-security-check --mock --verify  # CI / build-time smoke test
```

Checks:
- 11 kernel self-protection sysctls
- 9 network hardening sysctls
- 4 filesystem protection sysctls
- SSH config, nftables, AppArmor, dm-crypt, dm-verity
- Umask, coredump disable, zero-day scripts presence

---

## 7. Known Limitations (Pre-release)

- **Secure Boot (UEFI)**: BCM2837/Pi bootloader does not support UEFI Secure Boot.
  Kernel signature enforcement is applied via `lockdown=integrity` (prevents loading
  unsigned modules at runtime) as the available substitute.
- **dm-verity on root**: Requires split rootfs+hash-tree partition layout at image
  creation time. Full setup workflow via `zeroday-secureboot setup`.
- **AppArmor profiles**: Generic `apparmor-profiles` package installed; custom profiles
  for ZD-specific binaries (zeroday-crypt, zeroday-comp, etc.) are future work.
- **Bluetooth**: Service masked by default in security stage. Enable with
  `systemctl unmask bluetooth && systemctl start bluetooth` for BLE ops.

---

## Compliance Mapping

| Attack Category          | Mitigations Applied                                         |
|--------------------------|-------------------------------------------------------------|
| Memory corruption        | KASLR, ASLR, shadow call stack, init_on_alloc/free, SLAB hardening |
| Privilege escalation     | Yama ptrace, hidepid=2, AppArmor, SUID audit               |
| Information disclosure   | kptr_restrict=2, dmesg_restrict=1, vsyscall=none, strict_devmem |
| Rootkit / kernel mod     | lockdown=integrity, strict_module_rwx, kexec disabled       |
| Speculative execution    | PTI, KPTI, branch predictor hardening, EL2 vector hardening |
| Network exploitation     | nftables default-deny, SYN cookies, no ICMP redirect/source-route |
| Physical access          | LUKS2 encrypted vault, dm-verity rootfs integrity          |
| Lateral movement         | nftables forward=drop, rfkill block all (stealth default)  |
| Persistence              | IMA appraisal, dm-verity, audit framework                  |
