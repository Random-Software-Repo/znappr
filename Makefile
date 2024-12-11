
CXX = cargo build --release --target-dir target

PROGRAM = znappr

default:
	@echo "To compile run \"make build\" or run \"cargo build --release\"."

build:
	@if [ $$USER = root ];\
	then \
		echo "Do not run make to build $(PROGRAM) as root.\nInstalling with make as root is ok.";\
	else \
		$(CXX);\
	fi

clean: 
	rm -rf target

install:
	@cp target/release/$(PROGRAM) /usr/local/bin
	@chmod 755 /usr/local/bin/$(PROGRAM)
	@mkdir -p /usr/local/etc/znappr
	@chmod 755 /usr/local/etc/znappr
	@echo "Installing znappr does not install a configuration file (znappr.json)."
	@echo "A configuration file should be installed in /usr/local/etc/znappr/znappr.json,"
	@echo "or another location if using znappr -f <path to file>."
	@echo "Installing znappr does not run znappr. You should add a cron entry to run"
	@echo "znappr as needed."
