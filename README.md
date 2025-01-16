# znappr

Znappr is a zfs snapshot manager. Znapper will read a configuration file in
/usr/local/etc/znappr/znappr.json, or another location if the -f option is
specified. This configuration file will specify a number of jobs each of
which will instruct znappr to make and maintain (purge) snapshots of a
specific dataset.

When znappr purges snapshots, only snapshots of the specific dataset for
each job which match the labels specified for that job will be destroyed.
Any other snapshots (created by other jobs, other programs, or manually)
will be ignored.

## Building:

### Prerequisites:

-  You must have a rust toolchain installed prior to building weathr. You can install rust from:

> https://www.rust-lang.org/learn/get-started

-  Gnu Make (`make` on most if not all linux distributions, `gmake` on FreeBSD)

### To compile:

```
$ mkdir znappr-src
$ cd znappr-src
$ git clone https://www.github.com/Random-Software-Repo/printwrap
$ git clone https://www.github.com/Random-Software-Repo/znappr
$ cd znappr
$ make build
````

### To install:

From the cloned znappr repository:

```
$ sudo make install
````

## Running

Znappr is intended to be run via cron. When running from cron, the
frequency znappr is run should correspond to the most frequent job
specified in the configuration file. Usually once every 10 or 15
minutes should be sufficient. Once per hour will be enough if no jobs
require more.

A typical cron entry might look like this:
```
*/15  *  *  *  *    /usr/local/bin/znappr -f /path/to/znappr.json
```
To make znappr use UTC rather than the local time zone, add TZ=UTC before the command:
```
*/15  *  *  *  *    TZ=UTC /usr/local/bin/znappr -f /path/to/znappr.json
```
