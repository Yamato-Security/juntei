# Contributing to Juntei

Bug reports, fixes, documentation, tests, and new artifact support are welcome.
Keep pull requests focused, describe the forensic behavior being changed, and
include meaningful validation for parser changes. Do not upload private evidence,
credentials, or data You do not have permission to share.

## Contributor License Agreement

Before we merge contributed material, every relevant rights holder must accept
the [Juntei Contributor License Agreement](CLA.md) (CLA). This includes code,
documentation, tests, and artwork. A bug report that only describes a problem and
does not supply material for incorporation does not require a signature.

The CLA gives **Zachary Mathis** and **Shirofune Security Co., Ltd.** independent
permission to use and relicense Your Contributions, including in **paid,
proprietary, closed-source products and services**. You keep ownership of Your
work. Read the full agreement before signing.

The CLA selects Japanese law and the Tokyo District Court as the exclusive court
of first instance for disputes under the agreement, subject to mandatory
applicable law.

The CLA is separate from Juntei's public [RPL 1.5 license](LICENSE). It grants
additional rights to the named recipients; it does not change existing public
releases or remove third-party license obligations.

## How to sign

Sign once for version 1.0; it covers Your own past and future contributions to
this repository. Each contributor signs for their own rights. If You do not own
the relevant rights, an authorized representative of the rights holder must
sign, or provide written authorization that covers this agreement.

1. Open a pull request and read [CLA.md](CLA.md).
2. Open [`CLA.md` on `main`](https://github.com/Yamato-Security/juntei/blob/main/CLA.md),
   whichever branch Your pull request targets. If the version You are signing
   has not reached `main` yet, open it on `dev` instead. Press **Y** in GitHub to
   obtain its permalink, and copy that URL. It must identify a full commit SHA,
   not a moving branch name. Sign only a version that has been merged into
   `main` or `dev`; a copy or proposed change in a pull request, including Your
   own, is not sufficient.
3. Post the following statement as a new comment **from Your own GitHub account**.
   Replace the bracketed fields. Use the private process below if the identifying
   information should not be public.

```text
I have read and agree to the Juntei Contributor License Agreement, version 1.0,
at [full commit permalink to CLA.md]. I intend this comment as my electronic
signature. I have authority to grant the rights in that agreement, including
the right for Zachary Mathis and Shirofune Security Co., Ltd. to use and
relicense my covered contributions in paid, proprietary, closed-source products
and services. My acceptance covers my past and future contributions to
Yamato-Security/juntei to the extent I own or am authorized to license them.
I also agree to the Japanese governing law and exclusive first-instance
jurisdiction of the Tokyo District Court specified in the agreement, subject
to its mandatory-law qualification.

Legal name of the rights holder: [individual's full name or legal entity name]
Signer's full name: [full name]
GitHub account: @[username]
Capacity: [signing for myself, or role and authority to sign for the entity]
```

4. A maintainer verifies and archives the acceptance. On later pull requests,
   link the previously verified signature or ask the maintainer to check the
   retained record. Re-sign if the rights holder changes or if You agree to a
   different version.

**A PR-template checkbox, a `Signed-off-by` line, a maintainer's comment on Your
behalf, or opening a pull request alone is not the required acceptance.** For a
multi-author pull request, identify all contributors and their rights holders;
one person's signature cannot cover others' independently owned contributions.
Reviewers who submit patches must also be covered before those patches are used.

### Employer or corporate ownership

Obtain permission before signing when an employer or client owns Your work.
An authorized representative may use the same statement on behalf of the legal
entity and identify the contributors and contributions covered. Merely listing
a company name or having a company email address does not establish authority.

### Private acceptance

If Your legal name, corporate authorization, or signature must be private, ask a
maintainer to arrange a private signing channel. Do not post addresses, identity
documents, or confidential employment paperwork in a public issue or PR. The
maintainer must retain the signed agreement and confirm coverage on the PR
before merging. No signature service or CLA bot is currently configured.

## Third-party and earlier material

Disclose the source and license of copied or adapted material, libraries,
fixtures, datasets, and generated material that carries third-party terms.
Keep their copyright and license notices. The CLA does not give You authority
to relicense someone else's material.

Existing contributors are asked to sign for their own earlier submissions.
Until the relevant rights holders have agreed or another sufficient license is
established, their contributions remain governed by their existing licenses.
Do not assume that introducing a CLA clears the repository's entire history for
a proprietary edition.

## Before merging

Maintainers must follow the [CLA review and recordkeeping checklist](.github/CLA-MAINTAINERS.md).
It is a **manual merge requirement**, not an automated GitHub status check.
Do not merge contributed material until the necessary acceptances and any
third-party restrictions have been reviewed.
