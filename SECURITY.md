# Security Policy

## Reporting Security Vulnerabilities

If you discover a security vulnerability in NebGov, please report it responsibly and privately rather than using the public issue tracker.

### How to Report

Please email security concerns to the NebGov team. We take all security reports seriously and will work with you to address the issue.

When reporting a vulnerability, please include:
- A description of the vulnerability
- Steps to reproduce the issue (if applicable)
- Potential impact
- Suggested fix (if you have one)

### What to Expect

- We will acknowledge receipt of your report within 48 hours
- We will assess the severity and work on a fix
- We will keep you informed of our progress
- We will credit you in the security advisory once the issue is resolved (unless you prefer otherwise)

## Security Audits

NebGov contracts undergo security reviews and audits. Information about completed audits and known security considerations can be found in [docs/security.md](./docs/security.md).

## Security Best Practices

When deploying NebGov in production:

1. Review [docs/security.md](./docs/security.md) for treasury reentrancy analysis and security notes
2. Follow the [parameter guide](./docs/parameter-guide.md) for safe governance parameter ranges
3. Use the official [deployment guide](./docs/deployment.md) for production deployment
4. Keep all dependencies up to date
5. Test thoroughly in testnet before mainnet deployment

## Responsible Disclosure

We follow responsible disclosure practices. Please allow us time to address vulnerabilities before public disclosure. We will work to release patches as quickly as possible.

## License

This security policy is provided under the same license as the NebGov project (MIT).
