# Development workflow targets
install-tools:
	@echo "🔧 Installing development tools..."
	cargo install cargo-tarpaulin

format-check:
	@echo "🎨 Checking code formatting..."
	cargo fmt --all -- --check

format:
	@echo "🎨 Formatting code..."
	cargo fmt --all

clippy:
	@echo "📎 Running clippy on production code..."
	cargo clippy --workspace --lib --bins -- -D warnings
	@echo "✅ Production code clippy check passed"

test:
	@echo "🧪 Running unit and integration tests..."
	cargo test --workspace --all-features -- --nocapture

coverage:
	@echo "📊 Measuring code coverage..."
	cargo tarpaulin --timeout 240 --skip-clean --workspace --lib --out Xml
	@if [ -f "cobertura.xml" ]; then \
		COVERAGE=$$(sed -n 's/.*<coverage[^>]*line-rate="\([^"]*\)".*/\1/p' cobertura.xml); \
		if [ -n "$$COVERAGE" ]; then \
			COVERAGE_PERCENT=$$(echo "$$COVERAGE" | awk '{printf "%.2f", $$1 * 100}'); \
			echo "📊 Code coverage: $${COVERAGE_PERCENT}%"; \
			if [ $$(echo "$$COVERAGE < 0.8" | bc -l) -eq 1 ]; then \
				echo "❌ Code coverage below 80%"; \
				exit 1; \
			else \
				echo "✅ Code coverage is sufficient"; \
			fi; \
		else \
			echo "❌ Failed to extract coverage value"; \
			exit 1; \
		fi; \
	else \
		echo "❌ Coverage report not generated"; \
		exit 1; \
	fi

pre-commit: format-check clippy test coverage
	@echo "🚀 All pre-commit checks passed!"

# Run full development workflow
dev: install-tools pre-commit
	@echo "🎉 Development workflow completed successfully!"

# Quick check without coverage (faster)
check: format-check clippy test
	@echo "✅ Quick checks passed!"

.PHONY: install-tools format-check format clippy test coverage pre-commit dev check qa qa-unix qa-windows qa-validate qa-summary

# QA and smoke testing targets
qa:
	@echo "🔍 Running QA smoke tests with full output..."
	cargo test smoke -- --nocapture

qa-unix:
	@echo "🐧 Running Unix workflow smoke tests..."
	cargo test test_workflow_smoke_tests_unix -- --nocapture

qa-windows:
	@echo "🪟 Running Windows workflow smoke tests..."
	cargo test test_workflow_smoke_tests_windows -- --nocapture

qa-validate:
	@echo "✅ Validating all workflow YAML files..."
	cargo test test_workflow_file_validation -- --nocapture

qa-summary:
	@echo "📊 Running QA summary tests..."
	cargo test test_qa_workflow_summary
