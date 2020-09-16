.PHONY: test example

example: example-run

example-build:
	RUST_BACKTRACE=full RUST_LOG=debug cargo bundle --format ios --target x86_64-apple-ios --example ios-example

example-install:  example-build
	xcrun simctl install booted $(PWD)/../target/x86_64-apple-ios/debug/examples/bundle/ios/ios-example.app/

example-run:  example-install
	xcrun simctl launch --console booted com.github.iced.simple

test:
	cargo dinghy --platform auto-ios-x86_64 test

check:
	cargo check --target x86_64-apple-ios
