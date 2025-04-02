BINARY ?= berastribute
BUILD ?= .build
BALLS ?= .balls
TGTS ?= \
	x86_64-unknown-linux-gnu
TAG ?= v0.6.0
VER ?= v0_6_0
REL_NAME_PREFIX ?= $(BINARY)
REL_NAME_SUFFIX ?=
REL_DESC ?= automatic release from Makefile
GH_USER ?= p2p-org
GH_REPO ?= devops-tmbuilds-berastribute

.PHONY: all
all: install-deps build balls release

.PHONY: build
build:
	@/usr/bin/env bash -c ' \
		echo ">> building binaries…" ; \
		/usr/bin/env mkdir -vp "$(BUILD)"; \
		HASH=$$(git rev-parse HEAD | cut -b1-8 2>/dev/null) && \
		for tgt in $(TGTS); do \
			echo " > target [$(VER)-$$tgt]" && \
			cargo build --release --target $$tgt && \
			/usr/bin/env cp -f "target/$$tgt/release/$(BINARY)" \
				"$(BUILD)/$(BINARY)-$(VER)-$$tgt" ; \
		done ; \
	'

.PHONY: balls
balls:
	@/usr/bin/env bash -c ' \
		echo ">> building release balls…" ; \
		/usr/bin/env mkdir -vp "$(BALLS)" && \
		for tgt in $(TGTS); do \
			BALL_PFX="../$(BALLS)/$(BINARY)-$(VER)-$$tgt" && \
			echo " > target [$(VER)-$$tgt]" && \
			( \
				cd "$(BUILD)" && \
				/usr/bin/env tar --create \
					--file $${BALL_PFX}.tar \
					$(BINARY)-$(VER)-$$tgt && \
				/usr/bin/env pixz $${BALL_PFX}.tar && \
				/usr/bin/env mv -f $${BALL_PFX}.tpxz $${BALL_PFX}.txz ; \
			) \
		done ; \
	'

.PHONY: release
release: get-github-release
	@/usr/bin/env bash -c ' \
		echo ">> pushing binaries to github…" ; \
		if [[ -z "$${GITHUB_TOKEN}" ]]; then \
			echo "Undefined or empty GITHUB_TOKEN environment variable, giving up…"; \
			exit 1; \
		fi; \
		echo " > creating release [$(TAG)] draft" && \
		/usr/bin/env git tag -m '\'''\'' "$(TAG)" && \
		/usr/bin/env git push origin "$(TAG)" && \
		/usr/bin/env github-release release \
			--draft \
			-t "$(TAG)" \
			-u "$(GH_USER)" \
			-r "$(GH_REPO)" \
			-n "$(REL_NAME_PREFIX) $(TAG) $(REL_NAME_SUFFIX)" \
			-d "$(REL_DESC)" && \
		while ! /usr/bin/env github-release info \
			-t "$(TAG)" \
			-u "$(GH_USER)" \
			-r "$(GH_REPO)" 2>/dev/null; do \
			echo "  > waiting to release will be ready…" \
			/usr/bin/env sleep 1; \
		done && \
		for tgt in $(TGTS); do \
			echo "  > arch [$$tgt] uploading" && \
			/usr/bin/env github-release upload \
			-t "$(TAG)" \
			-u "$(GH_USER)" \
			-r "$(GH_REPO)" \
			-n "$(BINARY)-$(VER)-$$tgt.txz" \
			-f "$(BALLS)/$(BINARY)-$(VER)-$$tgt.txz" ; \
		done \
	'

.PHONY: get-github-release
get-github-release:
	@/usr/bin/env bash -c ' \
		if [[ \
			"$$( \
				/usr/bin/env github-release --version 2>&1 | \
					/usr/bin/env awk '\''{print $$2}'\'' \
			)" != '\''v0.10.0'\'' \
		]]; then \
			/usr/bin/env curl -L https://github.com/github-release/github-release/releases/download/v0.10.0/linux-amd64-github-release.bz2 | \
				/usr/bin/env bzip2 -d >"$${GOPATH}/bin/github-release" && \
				/usr/bin/env chmod 755 "$${GOPATH}/bin/github-release" ; \
		fi \
	'

.PHONY: clean
clean:
	@rm -rfv "$(BUILD)" "$(BALLS)"
	@cargo clean

.PHONY: install-deps
install-deps:
	@rustup target add x86_64-unknown-linux-gnu
	@rustup target add aarch64-unknown-linux-gnu
