BOARD_SPEC_CODE_GEN = cargo run --quiet --bin board-spec-code-generator --features board-spec-generator --release --

boardspecs:
	cd jumperless-types && $(BOARD_SPEC_CODE_GEN) ../boardspec/v4 ../jumperless-common/src/board/v4.rs
	cd jumperless-types && $(BOARD_SPEC_CODE_GEN) ../boardspec/v5 ../jumperless-common/src/board/v5.rs
	rustfmt jumperless-common/src/board/v4.rs
	rustfmt jumperless-common/src/board/v5.rs

.PHONY: boardspecs
