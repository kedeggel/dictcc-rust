# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --features cli --target $TARGET
    cross build --features cli --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --features cli --target $TARGET
    cross test --features cli --target $TARGET --release

    cross run --features cli --target $TARGET -- -d tests/database/test_database.txt --no-config --no-pager -t r "*"
    cross run --features cli --target $TARGET --release -- -d tests/database/test_database.txt --no-config --no-pager -t r "*"
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
