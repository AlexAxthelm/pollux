.PHONY: help build clean test test-jvm test-ios run-mac run-ios dev reload ios-build ios-sim lint lint-report coverage coverage-xml deps-update license-report

# Set IOS_SIM_ID to your simulator UUID (find with: xcrun simctl list devices)
IOS_SIM_ID ?= 4B05657F-089D-45A9-A6EC-7EB5D52A16AC

# Default target - show help
help:
	@echo "Pollux - Development Commands"
	@echo ""
	@echo "Building:"
	@echo "  make build          Build the project"
	@echo "  make clean          Clean build artifacts"
	@echo "  make test           Run all tests"
	@echo "  make test-jvm       Run JVM tests only"
	@echo "  make test-ios       Run iOS simulator tests only"
	@echo ""
	@echo "Running:"
	@echo "  make run-mac        Run macOS app (production)"
	@echo "  make dev            Run macOS app with hot reload"
	@echo "  make reload         Trigger hot reload manually"
	@echo "  make run-ios        Open iOS project in Xcode"
	@echo ""
	@echo "iOS Simulator:"
	@echo "  make ios-build      Build iOS framework"
	@echo "  make ios-sim        Run in iOS simulator (CLI)"
	@echo ""
	@echo "Quality:"
	@echo "  make lint           Run static analysis (Detekt)"
	@echo "  make lint-report    Run Detekt with HTML report"
	@echo "  make coverage       Generate HTML coverage report (Kover)"
	@echo "  make coverage-xml   Generate XML coverage report (Kover)"
	@echo "  make deps-update    Check for dependency updates"
	@echo "  make license-report Generate dependency license report"

# Build
build:
	./gradlew build -x linkReleaseFrameworkIosArm64 -x linkReleaseFrameworkIosSimulatorArm64

clean:
	./gradlew clean

test:
	./gradlew allTests

test-jvm:
	./gradlew jvmTest

test-ios:
	./gradlew iosSimulatorArm64Test

# macOS
run-mac:
	./gradlew runDistributable

dev:
	./gradlew hotRunJvm

reload:
	./gradlew reload

# iOS
run-ios:
	./gradlew linkDebugFrameworkIosSimulatorArm64
	open iosApp/iosApp.xcodeproj

ios-build:
	./gradlew linkDebugFrameworkIosSimulatorArm64

coverage:
	./gradlew koverHtmlReport
	@echo "Report: composeApp/build/reports/kover/html/index.html"

coverage-xml:
	./gradlew koverXmlReport

lint:
	./gradlew detekt

lint-report:
	./gradlew detekt --reports html:build/reports/detekt/detekt.html

deps-update:
	./gradlew dependencyUpdates

license-report:
	./gradlew generateLicenseReport --no-configuration-cache
	@echo "Report: build/reports/dependency-license/index.html"

ios-sim: ios-build
	@echo "Building and installing app..."
	xcodebuild -project iosApp/iosApp.xcodeproj \
		-scheme iosApp \
		-configuration Debug \
		-destination 'platform=iOS Simulator,id=$(IOS_SIM_ID)' \
		-derivedDataPath build/ios
	@echo "Launching in simulator..."
	xcrun simctl boot $(IOS_SIM_ID) 2>/dev/null || true
	open -a Simulator
	xcrun simctl install $(IOS_SIM_ID) \
		build/ios/Build/Products/Debug-iphonesimulator/pollux.app
	xcrun simctl launch --console $(IOS_SIM_ID) \
		com.alexaxthelm.pollux.pollux
