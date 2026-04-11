check: rust-all-checks swift-all-checks
test: rust-test
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
IOS_SWIFT_DIR   := iOS/SimpleCounter

swift-all-checks: swift-lint swift-format-check

swift-lint:
	swiftlint lint --strict $(IOS_SWIFT_DIR)

swift-format:
	swiftformat $(IOS_SWIFT_DIR)

swift-format-check:
	swiftformat --lint $(IOS_SWIFT_DIR)

# -- iOS --
XCODE_PROJECT   := iOS/SimpleCounter.xcodeproj
XCODE_SCHEME    := SimpleCounter

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

# Build the shared library as a Swift package using cargo-swift (requires v0.9.0)
package:
	cd shared && \
		cargo swift --version | grep -q '0.9.0' && \
		cargo swift package \
			--name Shared \
			--platforms ios \
			--lib-type static \
			--features uniffi && \
		rm -rf generated && \
		rm -rf ../iOS/generated/Shared && \
		mkdir -p ../iOS/generated/Shared && \
		cp -r Shared/* ../iOS/generated/Shared/ && \
		rm -rf Shared

# Rebuild the Xcode project from project.yml
generate-project:
	xcodegen --spec iOS/project.yml --project iOS

ios-build: typegen package generate-project

ios-xcodebuild: ios-build
	xcodebuild \
		-project $(XCODE_PROJECT) \
		-scheme $(XCODE_SCHEME) \
		-configuration Debug \
		-destination 'platform=iOS Simulator,id=$(SIM_ID)' \
		| xcpretty

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
        ios-build ios-xcodebuild ios-rebuild ios-dev ios-sim ios-clean \
        swift-clean xcode-clean swift-generated-clean regenerate
