<div align="center">

# D.R.A.F.T

### Document Research, Analysis, Formatting & Text-analysis

**A local-first AI Writing & Research Assistant for Academics.**

<br />

![Built With Rust](https://img.shields.io/badge/Built%20With-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tauri 2](https://img.shields.io/badge/Tauri%202-Desktop-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)
![React](https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB)
![Tiptap](https://img.shields.io/badge/Tiptap-Editor-000000?style=for-the-badge)
![Python](https://img.shields.io/badge/Python-Helpers-3776AB?style=for-the-badge&logo=python&logoColor=white)
![Bash](https://img.shields.io/badge/Bash-Automation-4EAA25?style=for-the-badge&logo=gnubash&logoColor=white)
![MIT License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

</div>

---

## What is DRAFT?

DRAFT is a desktop application for people who write serious documents with sources, structure, evidence, and style requirements.

It is built for research papers, literature reviews, reports, policy drafts, technical documents, and long-form writing where accuracy and formatting both matter.

> **Development status:** DRAFT is in active pre-1.0 development. The repository currently contains the initial application toolchain scaffold. The product capabilities below describe the intended application and are not yet production-ready features.

DRAFT brings four writing jobs into one workspace:

**Document Research** helps organize scholarly sources, citations, bibliographies, and a structured reference library.

**Analysis** helps examine meaning, argument strength, source reliability, factual support, and voice consistency.

**Formatting** helps keep the document aligned with citation styles, headings, layout rules, and document-ready output.

**Text-analysis** helps review grammar, syntax, tone, clarity, cohesion, and overall writing quality.

## What Does It Do?

Research writing is usually split across too many tools.

One tool manages references. Another checks grammar. Another formats citations. Another summarizes sources. Another exports the document. The writer is left stitching everything together by hand.

That creates avoidable problems:

- Sources get disconnected from claims.
- Citations drift away from bibliographies.
- Formatting breaks late in the process.
- AI output gets mixed with verified evidence.
- Long documents become hard to audit.
- Writers spend too much time managing the document instead of improving the argument.

DRAFT is designed to reduce that friction.

It gives the writer one place to research, reason, format, and review the text before the document leaves the workspace.

## Why Do I Need It?

A serious document is not just words on a page.

It is a chain of claims, sources, structure, and style rules. When that chain breaks, the document loses trust.

DRAFT cares about that chain.

It is designed around a simple idea: writing support should make the author more aware of the document, not less aware of it.

That means the app should help surface problems, preserve source context, show what changed, and keep human judgment in control. The goal is not to hide the work. The goal is to make the work easier to inspect.

## How DRAFT Works

DRAFT separates the writing surface from the trusted engine underneath it.

The visible editor is clean and interactive. It uses React and Tiptap so writers can work in a familiar document-style interface.

The trusted core is written in Rust. Rust owns the work that must be durable and safe: saving files, managing document state, handling background jobs, calling external services, storing secrets, and exporting documents.

Python is used for helper work where it makes sense, such as formatting checks or text-analysis routines. Those helpers do not own the document. They are called by the Rust core through controlled inputs and outputs.

Bash is used for developer automation, formatting commands, and GitHub Actions workflows. It supports the build and verification process; it is not the product runtime.

This structure keeps the app understandable. The user interface presents the work. The Rust core protects the work. Helper tools assist the work without taking ownership away from the core.

## What DRAFT Plans to Support

DRAFT is designed to support the full writing pipeline:

- Finding and organizing scholarly sources.
- Managing citation records and bibliographies.
- Checking whether claims are supported by sources.
- Reviewing argument structure and document flow.
- Enforcing APA, MLA, Chicago, and other style rules.
- Improving grammar, clarity, tone, and cohesion.
- Exporting document-ready files.
- Keeping AI-assisted work visible and reviewable.

DRAFT is not meant to replace the writer. It is meant to protect the writing process from drift, fragmentation, and silent errors.

## Built With Rust

DRAFT uses Rust as the trusted core because the core handles the parts of the application where mistakes are expensive.

Rust is responsible for document safety, filesystem access, network boundaries, background work, citation consistency, export paths, and worker orchestration.

The result is a desktop app shaped around reliability first, with a modern writing interface on top.

## Built with

- **Rust** for the trusted application core.
- **Tauri 2** for the desktop shell.
- **TypeScript** for safer frontend code.
- **React** for the user interface.
- **Tiptap** for the document editor.
- **Python** for controlled formatting and text-analysis helpers.
- **Bash** for local development and GitHub Actions automation.

## Development

The current scaffold requires Rust 1.96.0, Node.js 22.12.0 or newer with npm, Python 3.12 or newer, and Bash.

Bootstrap the locked dependency trees from the repository root:

```bash
bash scripts/bootstrap.sh
```

See `docs/maintainers/TOOLCHAIN.md` for the current toolchain scope and supported commands.

## License

DRAFT is released under the **MIT License**.

See `LICENSE` for the full license text.
