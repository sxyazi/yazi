#[cfg(test)]
mod security_tests {
    use std::path::{Path, PathBuf};
    use std::collections::HashMap;

    // Simulated representation of a package/plugin source entry
    #[derive(Debug, Clone)]
    struct PackageSource {
        url: String,
        commit: Option<String>,
        hash: Option<String>,
        signature: Option<String>,
    }

    impl PackageSource {
        fn new(url: &str) -> Self {
            PackageSource {
                url: url.to_string(),
                commit: None,
                hash: None,
                signature: None,
            }
        }

        fn with_commit(mut self, commit: &str) -> Self {
            self.commit = Some(commit.to_string());
            self
        }

        fn with_hash(mut self, hash: &str) -> Self {
            self.hash = Some(hash.to_string());
            self
        }

        fn with_signature(mut self, sig: &str) -> Self {
            self.signature = Some(sig.to_string());
            self
        }

        /// Security invariant: a package source must have integrity verification
        /// before it can be considered safe to install.
        fn has_integrity_verification(&self) -> bool {
            // Must have at least one form of integrity verification:
            // - A pinned commit hash (full SHA, not a branch/tag)
            // - A content hash
            // - A cryptographic signature
            let has_pinned_commit = self.commit.as_ref().map_or(false, |c| {
                // A proper pinned commit should be a full 40-char SHA1 or 64-char SHA256
                let trimmed = c.trim();
                let is_full_sha1 = trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit());
                let is_full_sha256 = trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit());
                is_full_sha1 || is_full_sha256
            });

            let has_content_hash = self.hash.as_ref().map_or(false, |h| {
                let trimmed = h.trim();
                !trimmed.is_empty() && trimmed.len() >= 32
            });

            let has_signature = self.signature.as_ref().map_or(false, |s| {
                !s.trim().is_empty()
            });

            has_pinned_commit || has_content_hash || has_signature
        }

        /// Security invariant: URL must not be a local path or use insecure protocols
        fn has_secure_transport(&self) -> bool {
            let url = self.url.trim();
            // Must not be empty
            if url.is_empty() {
                return false;
            }
            // Must not be a local filesystem path
            if url.starts_with('/') || url.starts_with("./") || url.starts_with("../") {
                return false;
            }
            // Must not use insecure git:// protocol (no TLS)
            if url.starts_with("git://") {
                return false;
            }
            // Must not use http:// (unencrypted)
            if url.starts_with("http://") {
                return false;
            }
            true
        }
    }

    /// Simulates what a secure package fetch configuration validator should enforce
    fn validate_package_for_installation(pkg: &PackageSource) -> Result<(), String> {
        if !pkg.has_secure_transport() {
            return Err(format!(
                "Package URL '{}' uses insecure transport or local path",
                pkg.url
            ));
        }
        if !pkg.has_integrity_verification() {
            return Err(format!(
                "Package '{}' lacks integrity verification (no pinned commit, hash, or signature)",
                pkg.url
            ));
        }
        Ok(())
    }

    #[test]
    fn test_package_fetch_requires_integrity_verification() {
        // Invariant: Any package source that lacks integrity verification
        // (pinned commit hash, content hash, or cryptographic signature)
        // MUST be rejected before installation. This prevents compromised
        // repositories from silently delivering malicious code.

        let adversarial_payloads: Vec<(PackageSource, &str)> = vec![
            // No integrity verification at all
            (
                PackageSource::new("https://github.com/evil/malicious-plugin"),
                "package with no integrity verification must be rejected",
            ),
            // Branch reference instead of pinned commit (mutable, can be changed by attacker)
            (
                PackageSource::new("https://github.com/user/plugin").with_commit("main"),
                "mutable branch reference must be rejected as integrity verification",
            ),
            // Tag reference (can be moved/deleted and recreated)
            (
                PackageSource::new("https://github.com/user/plugin").with_commit("v1.0.0"),
                "mutable tag reference must be rejected as integrity verification",
            ),
            // Short/abbreviated commit hash (ambiguous, collision-prone)
            (
                PackageSource::new("https://github.com/user/plugin").with_commit("abc123"),
                "abbreviated commit hash must be rejected as integrity verification",
            ),
            // Empty commit string
            (
                PackageSource::new("https://github.com/user/plugin").with_commit(""),
                "empty commit string must be rejected as integrity verification",
            ),
            // Whitespace-only commit
            (
                PackageSource::new("https://github.com/user/plugin").with_commit("   "),
                "whitespace-only commit must be rejected as integrity verification",
            ),
            // HEAD reference (always points to latest, attacker-controlled)
            (
                PackageSource::new("https://github.com/user/plugin").with_commit("HEAD"),
                "HEAD reference must be rejected as integrity verification",
            ),
            // Insecure git:// protocol (no TLS, MITM possible)
            (
                PackageSource::new("git://github.com/user/plugin")
                    .with_commit("abc123def456abc123def456abc123def456abc1"),
                "git:// protocol must be rejected due to lack of transport security",
            ),
            // Insecure http:// protocol
            (
                PackageSource::new("http://github.com/user/plugin")
                    .with_commit("abc123def456abc123def456abc123def456abc1"),
                "http:// protocol must be rejected due to lack of transport security",
            ),
            // Local filesystem path (could be attacker-controlled)
            (
                PackageSource::new("/tmp/malicious-plugin")
                    .with_commit("abc123def456abc123def456abc123def456abc1"),
                "local filesystem path must be rejected",
            ),
            // Relative path traversal
            (
                PackageSource::new("../../etc/passwd"),
                "path traversal must be rejected",
            ),
            // Empty URL
            (
                PackageSource::new(""),
                "empty URL must be rejected",
            ),
            // Short hash (too short to be a real commit)
            (
                PackageSource::new("https://github.com/user/plugin").with_hash("abc"),
                "too-short hash must be rejected as integrity verification",
            ),
        ];

        for (payload, description) in &adversarial_payloads {
            let result = validate_package_for_installation(payload);
            assert!(
                result.is_err(),
                "SECURITY VIOLATION: {} — package was accepted but should have been rejected. URL: '{}', commit: {:?}, hash: {:?}",
                description,
                payload.url,
                payload.commit,
                payload.hash
            );
        }
    }

    #[test]
    fn test_package_fetch_accepts_properly_verified_sources() {
        // Invariant: Packages with proper integrity verification and secure transport
        // should be accepted. This ensures the security check doesn't break legitimate use.

        let valid_payloads: Vec<(PackageSource, &str)> = vec![
            // Full SHA1 pinned commit
            (
                PackageSource::new("https://github.com/user/plugin")
                    .with_commit("abc123def456abc123def456abc123def456abc1"),
                "full SHA1 pinned commit with https should be accepted",
            ),
            // Full SHA256 pinned commit
            (
                PackageSource::new("https://github.com/user/plugin")
                    .with_commit("abc123def456abc123def456abc123def456abc123def456abc123def456abcd"),
                "full SHA256 pinned commit with https should be accepted",
            ),
            // Content hash provided
            (
                PackageSource::new("https://github.com/user/plugin")
                    .with_hash("sha256:abc123def456abc123def456abc123def456abc123def456abc123def456abcd"),
                "package with content hash should be accepted",
            ),
            // Signature provided
            (
                PackageSource::new("https://github.com/user/plugin")
                    .with_signature("-----BEGIN PGP SIGNATURE-----\nvalidsignaturedata\n-----END PGP SIGNATURE-----"),
                "package with cryptographic signature should be accepted",
            ),
            // SSH URL with pinned commit (secure transport)
            (
                PackageSource::new("git@github.com:user/plugin.git")
                    .with_commit("abc123def456abc123def456abc123def456abc1"),
                "SSH URL with pinned commit should be accepted",
            ),
        ];

        for (payload, description) in &valid_payloads {
            let result = validate_package_for_installation(payload);
            assert!(
                result.is_ok(),
                "Legitimate package was incorrectly rejected: {} — URL: '{}', error: {:?}",
                description,
                payload.url,
                result.err()
            );
        }
    }

    #[test]
    fn test_commit_hash_pinning_invariant() {
        // Invariant: Only full-length cryptographic commit hashes should be
        // accepted as integrity verification. Mutable references (branches, tags,
        // HEAD) must always be rejected regardless of other properties.

        let mutable_refs = vec![
            "main",
            "master",
            "develop",
            "HEAD",
            "HEAD~1",
            "HEAD^",
            "v1.0",
            "v1.0.0",
            "latest",
            "stable",
            "release",
            "abc123",        // too short
            "abc123def456",  // still too short
            "",
            "   ",
            "\n",
            "\t",
            "refs/heads/main",
            "refs/tags/v1.0",
            "origin/main",
        ];

        for mutable_ref in &mutable_refs {
            let pkg = PackageSource::new("https://github.com/user/plugin")
                .with_commit(mutable_ref);

            assert!(
                !pkg.has_integrity_verification(),
                "SECURITY VIOLATION: Mutable or invalid ref '{}' was accepted as integrity verification. \
                 This allows attackers to substitute malicious code by updating the reference.",
                mutable_ref
            );
        }

        // Full SHA1 hashes must be accepted
        let valid_sha1 = "abc123def456abc123def456abc123def456abc1";
        let pkg_sha1 = PackageSource::new("https://github.com/user/plugin")
            .with_commit(valid_sha1);
        assert!(
            pkg_sha1.has_integrity_verification(),
            "Full SHA1 commit hash should be accepted as integrity verification"
        );

        // Full SHA256 hashes must be accepted
        let valid_sha256 = "abc123def456abc123def456abc123def456abc123def456abc123def456abcd";
        let pkg_sha256 = PackageSource::new("https://github.com/user/plugin")
            .with_commit(valid_sha256);
        assert!(
            pkg_sha256.has_integrity_verification(),
            "Full SHA256 commit hash should be accepted as integrity verification"
        );
    }

    #[test]
    fn test_no_silent_integrity_bypass_via_multiple_fields() {
        // Invariant: Having an insecure transport must always fail validation,
        // even if integrity verification fields are present. Security checks
        // must not be bypassable by providing additional fields.

        let insecure_but_verified = vec![
            PackageSource::new("git://github.com/user/plugin")
                .with_commit("abc123def456abc123def456abc123def456abc1"),
            PackageSource::new("http://github.com/user/plugin")
                .with_commit("abc123def456abc123def456abc123def456abc1")
                .with_hash("sha256:abc123def456abc123def456abc123def456abc123def456abc123def456abcd"),
            PackageSource::new("/local/path/plugin")
                .with_commit("abc123def456abc123def456abc123def456abc1")
                .with_signature("valid-signature"),
        ];

        for pkg in &insecure_but_verified {
            let result = validate_package_for_installation(pkg);
            assert!(
                result.is_err(),
                "SECURITY VIOLATION: Package with insecure transport '{}' was accepted \
                 despite having integrity verification fields. Insecure transport must \
                 always be rejected to prevent MITM attacks.",
                pkg.url
            );
        }
    }
}