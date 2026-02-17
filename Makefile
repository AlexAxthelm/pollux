.PHONY: help build clean test run-mac run-ios dev reload

# Default target - show help
help:
	@echo "Pollux - Development Commands"
	@echo ""
	@echo "Building:"
	@echo "  make build          Build the project"
	@echo "  make clean          Clean build artifacts"
	@echo "  make test           Run tests"
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

# Build
build:
	./gradlew build

clean:
	./gradlew clean

test:
	./gradlew test

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

ios-sim:
	@echo "Building iOS framework..."
	./gradlew linkDebugFrameworkIosSimulatorArm64
	@echo "Building and installing app..."
	xcodebuild -project iosApp/iosApp.xcodeproj \
		-scheme iosApp \
		-configuration Debug \
		-destination 'platform=iOS Simulator,id=4B05657F-089D-45A9-A6EC-7EB5D52A16AC' \
		-derivedDataPath build/ios
	@echo "Launching in simulator..."
	xcrun simctl boot 4B05657F-089D-45A9-A6EC-7EB5D52A16AC 2>/dev/null || true
	xcrun simctl install 4B05657F-089D-45A9-A6EC-7EB5D52A16AC \
		build/ios/Build/Products/Debug-iphonesimulator/pollux.app
	xcrun simctl launch --console 4B05657F-089D-45A9-A6EC-7EB5D52A16AC \
		com.alexaxthelm.pollux.pollux
