.PHONY: xcode cargo run install simulator-logs

LIB = rust/libiced_ios_example.a
run: install
	xcrun simctl launch booted com.iced.IcedExample

install: xcode
	xcrun simctl install booted build/Build/Products/Debug-iphonesimulator/IcedExample.app

xcode: $(LIB)
	xcrun xcodebuild -scheme IcedExample -project IcedExample.xcodeproj/ -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 11 Pro,OS=13.2' -derivedDataPath build

cargo:
	make -C rust

simulator-logs:
	xcrun simctl spawn booted log stream --level=debug --predicate 'processImagePath endswith "IcedExample"'
ci:
	make -C rust ci
	xcrun xcodebuild -scheme IcedExample -project IcedExample.xcodeproj/ -configuration Debug -destination 'platform=iOS Simulator,name=iPhone 11 Pro,OS=13.2' -derivedDataPath build


