# Documentation

[简体中文](README.zh-CN.md) | English

This directory contains the maintained documentation set. Historical bring-up journals and
duplicated platform notes were consolidated so each topic has one authoritative English document
and one Simplified Chinese counterpart.

- [Architecture](ARCHITECTURE.md): components, trust boundaries, and cross-platform design.
- [Microdroid implementation](MICRODROID.md): detailed Linux, macOS, and Windows guest,
  virtualization, payload, transport, security, and feature-alignment review.
- [Full Android implementation](ANDROID.md): detailed Cuttlefish-derived artifacts, graphics,
  virtual devices, platform parity, source changes, and release evidence.
- [Deployment](DEPLOYMENT.md): checkout, prerequisites, build, staging, and release layout.
- [Security](SECURITY.md): threat model, isolation guarantees, limitations, and hardening.
- [Operations](OPERATIONS.md): supported scripts, validation sequence, logs, and troubleshooting.
- [Cuttlefish compatibility](CUTTLEFISH.md): concise optional full Android workflow.

The root [README](../README.md) is the repository overview. The manifest repository README remains
the primary entry point for a complete multi-repository checkout.
