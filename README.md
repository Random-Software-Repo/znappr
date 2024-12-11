# znappr

     Znappr is a zfs snapshot manager. Znapper will read a configuration file in /usr/local/etc/znappr/znappr.json, or another location if the -f option is specified. This configuration file will specify a number of jobs each of which will instruct znappr to make and maintain (purge) snapshots of a specific dataset.

     When znappr purges snapshots, only snapshots of the specific dataset for each job which match the labels specified for that job will be destroyed. Any other snapshots (created by other jobs, other programs, or manually) will be ignored.

     Building:
         Prerequisites:
             Rust 1.63 or newer
             Both the znappr and printwrap repos from random-software-repo:
                  git clone https://www.github.com/Random-Software-Repo/znappr
                  git clone https://www.github.com/Random-Software-Repo/printwrap
             Gnu Make (make on most if not all linux distrobutions, gmake on FreeBSD)
     To compile, run:
          make build
        or
          cargo build --release
     To install, run:
          sudo make install
        or
          sudo make install dir=/an/alternate/path/for/znappr

     Znappr is intended to be run via cron. When running from cron, the frequency znappr is run should correspond to the most frequent job specified in the configuration file. Usually once every 10 or 15 minutes should be sufficient. Once per hour will be enough if no jobs require more. 
     A typical cron entry might look like this:
         */15  *  *  *  *    /usr/local/bin/znappr -f /path/to/znappr.json
     To make znappr use UTC rather than the local time zone, invoke thusly:
         */15  *  *  *  *    TZ=UTC /usr/local/bin/znappr -f /path/to/znappr.json
