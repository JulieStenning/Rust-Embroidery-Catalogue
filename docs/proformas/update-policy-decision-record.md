## Update Policy Decision Record (Fill Before First Live Update)

Use this proforma to approve and record your release/update policy before the first live update.
For release type definitions and migration scope defaults, refer to docs/policies/releases/release-types-and-migration-scope.md.

- Date approved: [YYYY-MM-DD]
- Approved by (Release Owner): [Name]
- Approved by (Verifier): [Name]
- Policy version: [v1.0]

1. Update scope.
2. Channels included now: [Installer only / Portable only / Installer + Portable]
3. Rollout audience: [Internal testers / All users / Phased]
4. Supported Windows versions/architectures: [e.g., Win10/11 x64]

5. Versioning and change policy.
6. Hotfix definition: [criteria]
7. Minor definition: [criteria]
8. Major definition: [criteria]
9. Breaking changes allowed in: [Major only / Other]
10. Database migrations allowed in: [Hotfix/Minor/Major]

11. Compatibility promise.
12. Upgrade support window: [e.g., current and one previous release]
13. Skipping versions supported: [Yes/No]
14. Sequential update required: [Yes/No]

15. Data safety policy.
16. Backup before update: [Mandatory / Strongly recommended]
17. Protected data locations during update: [list paths]
18. Custom data-root preservation requirement: [rule]

19. Migration and failure handling.
20. Migration trigger point: [startup/install/first launch]
21. Migration failure behavior: [block launch + show recovery steps]
22. Downgrade migrations supported: [Yes/No]
23. If no downgrade, rollback method: [backup restore + known-good installer]

24. Rollback policy.
25. Rollback trigger criteria: [critical severity conditions]
26. Max time-to-advisory target: [e.g., 2 hours]
27. Known-good artifact retention rule: [where/how long]

28. Quality gates policy.
29. Mandatory pre-release test suites: [list per selected channel]
30. Mandatory scenario tests: [installer: existing-data, clean machine, custom data-root; portable: existing-data media, clean media, path portability]
31. Gate override allowed: [Yes/No]
32. If override allowed, required approvals: [roles]

33. Security and trust policy.
34. Signing requirement by release type: [tester/prod rules]
35. Checksum publication required: [Yes/No]
36. Secret handling constraints for build artifacts: [rules]

37. Release communication policy.
38. Required release note sections: [list]
39. Mandatory user instruction: [backup-before-update wording]
40. Support triage location and SLA: [where/how fast]

41. Operational ownership policy.
42. Release GO/NO-GO decision owner: [role]
43. Early monitoring owner (24-48h): [role]
44. Pause/hotfix/rollback decision owner: [role]

45. Evidence requirements.
46. Required evidence package for each release: [versions, tests, migration logs, checksum, signing proof, release URL]
47. Evidence storage location: [path/tool]

48. Policy sign-off.
49. Release Owner signature: [Name, date]
50. Verifier signature: [Name, date]
51. Next policy review date: [YYYY-MM-DD]
