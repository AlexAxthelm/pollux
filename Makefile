check: rust-all-checks swift-all-checks
test: rust-test ios-test
lint: rust-lint swift-lint
format-check: rust-format-check swift-format-check
format: rust-format swift-format

clean: rust-clean swift-clean

# -- Rust --

rust-all-checks: rust-check rust-test rust-lint rust-format-check rust-lock-check

rust-check:
	cargo check

rust-test:
	cargo test

rust-lint:
	cargo clippy -- -D warnings

rust-format:
	cargo fmt

rust-format-check:
	cargo fmt -- --check

rust-lock-check:
	cargo check --locked

rust-build:
	cargo build

rust-clean:
	cargo clean

# -- Swift --
IOS_SWIFT_DIR   := iOS/Pollux

swift-all-checks: swift-lint swift-format-check

swift-lint:
	swiftlint lint --strict $(IOS_SWIFT_DIR)

swift-format:
	swiftformat $(IOS_SWIFT_DIR)

swift-format-check:
	swiftformat --lint $(IOS_SWIFT_DIR)

# -- iOS --
XCODE_PROJECT   := iOS/Pollux.xcodeproj
XCODE_SCHEME    := Pollux

SIM_DEVICE_NAME := iPhone 14 Pro Max
SIM_ID := $(shell \
	xcrun simctl list devices available -j \
	| jq -r '.devices \
		| to_entries[].value[] \
		| select(.name == "$(SIM_DEVICE_NAME)" and .isAvailable == true) \
		| .udid' \
	| head -1)

# Generate Swift types from the shared Rust core via the codegen binary
typegen:
	RUST_LOG=info cargo run \
		--package shared \
		--bin codegen \
		--features codegen,facet_typegen \
		-- \
			--language swift \
			--output-dir iOS/generated

# Build the shared library as a Swift package using cargo-swift.
# cargo-swift is pinned in the root Cargo.toml ([workspace.metadata.bin]) and run
# via cargo-run-bin, which builds it into a project-local .bin/ cache on first use.
# `cargo bin` injects the `swift` subcommand automatically (the cargo- prefix), so
# the command is `cargo bin cargo-swift package`, not `... swift package`.
package:
	cd shared && \
		cargo bin cargo-swift package \
			--name Shared \
			--platforms ios \
			--lib-type static \
			--features uniffi && \
		rm -rf generated && \
		rm -rf ../iOS/generated/Shared && \
		mkdir -p ../iOS/generated/Shared && \
		cp -r Shared/* ../iOS/generated/Shared/ && \
		rm -rf Shared
	# Flatten the xcframework headers. cargo-swift 0.9.0 nests the FFI modulemap in
	# a per-framework subdir (headers/RustFramework/module.modulemap), but the
	# Info.plist HeadersPath is "Headers", so SwiftPM never finds the modulemap and
	# `canImport(sharedFFI)` fails — the generated bindings then can't see RustBuffer
	# et al. Move the modulemap + header up to the headers root. (Fixed upstream in
	# cargo-swift 0.11.0 / PR #87; we stay on 0.9.0 to match uniffi 0.29.4.)
	for h in iOS/generated/Shared/RustFramework.xcframework/*/headers; do \
		if [ -d "$$h/RustFramework" ]; then \
			mv "$$h/RustFramework/"* "$$h/" && rmdir "$$h/RustFramework"; \
		fi; \
	done

# Rebuild the Xcode project from project.yml
generate-project: typegen package
	xcodegen --spec iOS/project.yml --project iOS

ios-build: generate-project

ios-xcodebuild: ios-build
	xcodebuild \
		-project $(XCODE_PROJECT) \
		-scheme $(XCODE_SCHEME) \
		-configuration Debug \
		-destination 'platform=iOS Simulator,id=$(SIM_ID)' \
		| xcbeautify

# Full rebuild from scratch
ios-rebuild: ios-clean ios-build

ios-dev: ios-build
	xed iOS

ios-sim: ios-xcodebuild
	$(eval APP_PATH := $(shell find ~/Library/Developer/Xcode/DerivedData \
		-name "$(XCODE_SCHEME).app" \
		-path "*/Debug-iphonesimulator/*" \
		-not -path "*/Index.noindex/*" \
		2>/dev/null | head -1))
	@[ -n "$(APP_PATH)" ] || \
		{ echo "App not found in DerivedData."; exit 1; }
	@[ -n "$(SIM_ID)" ] || \
		{ echo "Simulator '$(SIM_DEVICE_NAME)' not found."; exit 1; }
	@echo "Targeting: $(SIM_DEVICE_NAME) ($(SIM_ID))"
	@echo "Installing: $(APP_PATH)"
	xcrun simctl boot $(SIM_ID) 2>/dev/null || true
	open -a Simulator
	xcrun simctl install $(SIM_ID) "$(APP_PATH)"
	xcrun simctl launch --console $(SIM_ID) \
		$$(/usr/libexec/PlistBuddy -c "Print CFBundleIdentifier" "$(APP_PATH)/Info.plist")

ios-test: ios-build
	xcodebuild test \
		-project $(XCODE_PROJECT) \
		-scheme PolluxTests \
		-destination 'platform=iOS Simulator,id=$(SIM_ID)' \
		| xcbeautify

swift-clean: xcode-clean swift-generated-clean

xcode-clean:
	rm -rf $(XCODE_PROJECT)

ios-clean: xcode-clean swift-generated-clean

swift-generated-clean:
	rm -rf iOS/generated

# Wipe generated Swift types and regenerate
regenerate: swift-generated-clean typegen

.PHONY: check test lint format format-check clean \
        rust-all-checks rust-check rust-test rust-lint \
        rust-format rust-format-check rust-lock-check rust-build rust-clean \
        swift-all-checks swift-lint swift-format swift-format-check \
        typegen package generate-project \
        ios-build ios-xcodebuild ios-rebuild ios-dev ios-sim ios-test ios-clean \
        swift-clean xcode-clean swift-generated-clean regenerate
