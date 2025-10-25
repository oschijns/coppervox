default:
    @echo CopperVox

build:
    cd rust && cargo build --target-dir=target

build-release:
    cd rust && cargo build --target-dir=target --release
