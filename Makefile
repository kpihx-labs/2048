SHELL := /bin/zsh
.DEFAULT_GOAL := help

.PHONY: help fmt fmt-check lint test build smoke check install uninstall push tag push-tags release status log

help:  ## Show available targets
	@perl -ne 'print "  $$1\t$$2\n" if /^([a-zA-Z0-9_-]+):.*?##\s*(.*)$$/' $(MAKEFILE_LIST)

fmt:  ## Format Rust sources
	cargo fmt

fmt-check:  ## Verify Rust formatting
	cargo fmt --check

lint:  ## Run Clippy with warnings denied
	cargo clippy --all-targets --all-features -- -D warnings

test:  ## Run all unit, integration, and documentation tests
	cargo test --all-targets --all-features

build:  ## Build the optimized native binary
	cargo build --release --locked

smoke:  ## Verify non-interactive CLI entry points
	cargo run --locked -- --help
	cargo run --locked -- --version

check: fmt-check lint test build smoke  ## Run the complete project quality gate

install: check  ## Install the native 2048 binary through Cargo
	cargo install --path . --force --locked

uninstall:  ## Remove the Cargo-installed binary
	cargo uninstall terminal-2048

push:  ## Push the current branch to GitHub and GitLab
	@branch="$$(git branch --show-current)"; \
	for remote in github gitlab; do git push "$$remote" "$$branch"; done

tag: check  ## Create the annotated tag matching Cargo.toml
	@version="$$(perl -ne 'print $$1 if /^version = "([^"]+)"/' Cargo.toml)"; \
	git tag -a "v$$version" -m "2048 $$version"

push-tags:  ## Push release tags to GitHub and GitLab
	@for remote in github gitlab; do git push "$$remote" --tags; done

release: check push tag push-tags  ## Check, push, tag, and publish the release refs

status:  ## Show repository and remote status
	git status --short --branch
	git remote -v

log:  ## Show the recent project history
	git log --oneline -10
