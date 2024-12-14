extern crate printwrap;
use std::{process,process::Command,env,fs::File,path::Path, process::Stdio,str};
use log::*;
use chrono::{DateTime,Weekday,Local,Datelike,Timelike,Duration,NaiveDateTime,offset::TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PurgeRule
{
	unit: String,
	value: String
}
#[derive(Serialize, Deserialize)]
struct When
{
	field: String,
	value: String
}

#[derive(Serialize, Deserialize)]
struct Job 
{
	// SEE CONFIG_FILE_FORMAT.txt FOR A MODERATELY COMPLETE EXPLANATION OF THE CONFG FORMAT
	comment: Vec<String>,
	dataset: String,
	recursive: bool,
	prefix: String,
	postfix: String,
	date_format: String,
	pre_date: bool,
	whens: Vec<When>,
	purge_rule: PurgeRule,
}

#[derive(Serialize, Deserialize)]
struct Znappr
{
	jobs: Vec<Job>,
}
struct Snapshot
{
	snapshot: String,
	date: String
}
fn usage()
{
	printwrap::print_wrap(5,0,"Usage:");
	printwrap::print_wrap(5,0,"\tznappr [options]");
	printwrap::print_wrap(5,0,"Options:");
	printwrap::print_wrap(5,24,"    -f <config file>    Load the specified JSON config file. The default config file is loaded from: /usr/local/etc/znappr/znappr.json");
	printwrap::print_wrap(5,24,"    -s | --stdout       Log messages to stdout rather than syslog.");
	printwrap::print_wrap(5,24,"    -c | --configtest   Validate the config json file then exit.");
	printwrap::print_wrap(5,24,"    -h | --help         Print this usage information and exit.");
	printwrap::print_wrap(5,24,"    -j                  Prints an extended explanation of znappr's json configuration file format. This will not be legible on terminals less than 100 characters wide.");
	printwrap::print_wrap(5,24,"    -p                  Prints a generic configuration file for your local root (mountpoint=/) and home (mountpoint=/home) file systems that may be a useful starting point. If either of those filesystems don't exist, no configuration will be generated.");
	printwrap::print_wrap(5,24,"    -v | -vv            Increase the level of messaging by one or two levels (the maximum).");
	printwrap::print_wrap(5,24,"    --date <date>       Use the supplied date rather than the real date. <date> must be in for format: YYYY-MM-DD-HH:mm");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Znappr is a zfs snapshot manager. Znapper will read a configuration file in /usr/local/etc/znappr/znappr.json, or another location if the -f option is specified. This configuration file will specify a number of jobs each of which will instruct znappr to make and maintain (purge) snapshots of a specific dataset.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"When znappr purges snapshots, only snapshots of the specific dataset for each job which match the labels specified for that job will be destroyed. Any other snapshots (created by other jobs, other programs, or manually) will be ignored.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Znappr is intended to be run via cron. When running from cron, the frequency znappr is run should correspond to the most frequent job specified in the configuration file. Usually once every 10 or 15 minutes should be sufficient. Once per hour will be enough if no jobs require more. A typical cron line might look like this:");
	printwrap::print_wrap(5,0,"    */15  *  *  *  *    /usr/local/bin/znappr -f /path/to/znappr.json");
	printwrap::print_wrap(5,0,"To make znappr use UTC rather than the local time zone, invoke thusly:");
	printwrap::print_wrap(5,0,"    */15  *  *  *  *    TZ=UTC /usr/local/bin/znappr -f /path/to/znappr.json");
	printwrap::print_wrap(5,0,"");
	process::exit(1);

}

fn config_format()
{
	printwrap::print_wrap(5,0,"Znappr Configuration File");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"The znappr configuration is a text file with structured data in the json format.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"All fields are required. The structure looks like this:");
	printwrap::print_wrap(5,0,"<beginning of the config file>");
	printwrap::print_wrap(5,0,"{");
	printwrap::print_wrap(5,0,"    \"comment\":[\"This is a description of the overall\",");
	printwrap::print_wrap(5,0,"            \"contents of this file. All comment\",");
	printwrap::print_wrap(5,0,"            \"fields can be multiple lines in this\",");
	printwrap::print_wrap(5,0,"            \"manner. One quote delineated string\",");
	printwrap::print_wrap(5,0,"            \"followed, if desired, by a comma and\",");
	printwrap::print_wrap(5,0,"            \"additional quote delineated strings.\"],");
	printwrap::print_wrap(5,53,"    \"jobs\": [                                        Snapshot jobs. Each job will represent one \"rule\" to make snapshots of a specific dataset at specific frequencies. Each job will also include a rule to purge/prune older snapshots.");
	printwrap::print_wrap(5,53,"    {                                                First job (first or last makes no difference)");
	printwrap::print_wrap(5,53,"        \"comment\": [\"Description of this job\"],      Can be multiple lines as the above \"comment\"");
	printwrap::print_wrap(5,53,"        \"dataset\": \"/zroot/ROOT/root\",               The dataset on which to make snapshots. This dataset *MUST* exist.");
	printwrap::print_wrap(5,53,"        \"recursive\":true,                            Whether or not the snapshots should be recursive (true will include all child-dataset, false will not)");
	printwrap::print_wrap(5,58,"                                                       NOTE: When snapshots of child datasets are purged (see purge_rule below) the creation dates and counts of those snapshots are made at the parent level. If manual deletion of snapshots are made, the calculations for which snapshots should be delted will be affected.");
	printwrap::print_wrap(5,53,"        \"prefix\": \"QUARTER__\",                       This is prepended to the beginning of each snapshot. This is used to easily distinguish snapshots made by different snapshot jobs. The Prefix can *ONLY* be unicode alphabetic, unicode numeric (https://www.unicode.org/reports/tr44) or ascii dot (.), underscore (_) or hypen (-).");
	printwrap::print_wrap(5,53,"        \"postfix\": \"_NOT_REQUIRED\",                  This will be appended to the end of each snapshot. This is for additional infomational purposes as you desire. The Postfix can *ONLY* be unicode alphabetic, unicode numeric (see below) or ascii dot (.), underscore (_) or hypen (-).");
	printwrap::print_wrap(5,53,"        \"date_format\":\"%Y-%m-%d-%H:%M\",              Snapshot date format. Each snapshot will be tagged with the prefix + a date + postfix. The format of the date portion is specified here. This value must be compatible with rust's chrono::format (see below). There is no \"default\", but a good default value to use is: \"%Y-%m-%d-%H:%M\" The date will be in the local time zone unless znappr is started with a different timezone. On Linux and FreeBSD (at least) znappr can be started using UTC by prepending \"TZ=UTC\" before the znappr command. example:");
	printwrap::print_wrap(5,53,"                                                             TZ=UTC znappr [options...]");
	printwrap::print_wrap(5,53,"            pre_date: false,                         Predate is unusual. If predate is false, formatting of the snapshot's label's date portion will look exactly as anyone would expect. If pre_date is true, the date portion will be quite different: the date used for the tag will be the current date-time minus 1 hour. The time portion of this altered date will always be 00:00. This oddity is useful when a snapshot is made at midnight but the \"display\" date should be the previous day, such as an end-of-day/week/month/year snapshot. For example, a snapshot made at the end of the year 2025, scheduled to run at midnight would likely have a date of 2026-01-01, which might not be what one would want if human readability is valued. With ");
	printwrap::print_wrap(5,53,"                                                         prefix:\"YEARLY____\"");
	printwrap::print_wrap(5,53,"                                                         pre_date:true,");
	printwrap::print_wrap(5,53,"                                                         date_format:\"%Y\"");
	printwrap::print_wrap(5,53,"                                                     this would result in the 2025 year end snapshot having a tag/name of:");
	printwrap::print_wrap(5,53,"                                                         YEARLY____2025");
	printwrap::print_wrap(5,53,"                                                     Similarly, a job scheduled to run on the 1st of each month with");
	printwrap::print_wrap(5,53,"                                                         prefix:\"MONTHLY___\"");
	printwrap::print_wrap(5,53,"                                                         pre_date:true,");
	printwrap::print_wrap(5,53,"                                                         date_format:\"%Y-%m-%d\"");
	printwrap::print_wrap(5,53,"                                                     would have a shapshot made 2026-02-01 tagged \"MONTHLY___2026-01-31\" and so on.");
	printwrap::print_wrap(5,53,"        \"whens\":                                     Whens are one or more rules specifying when this job should be run. There can be more than one when rule in a job (and often will be). Each when condition can only represent one type (see field below), but you do not need to specity them all indivudually. Any fields that are omitted will be considered to be an asterisk for \"any/all\" values of that field.");
	printwrap::print_wrap(5,53,"                                                     When fields are *nearly* cron compatible frequency specifiers. Each when block includes two properties: field and value. field can be one of:");
	printwrap::print_wrap(5,53,"                                                          minute");
	printwrap::print_wrap(5,53,"                                                          hour");
	printwrap::print_wrap(5,53,"                                                          day");
	printwrap::print_wrap(5,53,"                                                          day_of_week");
	printwrap::print_wrap(5,53,"                                                          day_of_year");
	printwrap::print_wrap(5,53,"                                                          month");
	printwrap::print_wrap(5,53,"                                                     value for all fields indicates at which occurence of field the job should run. For all fields, value can be one of:");
	printwrap::print_wrap(5,53,"                                                          an asterisk, \"*\" (meaning all/any)");
	printwrap::print_wrap(5,53,"                                                          a single number");
	printwrap::print_wrap(5,53,"                                                          a comma separated list");
	printwrap::print_wrap(5,53,"                                                          a range <number>-<number>");
	printwrap::print_wrap(5,53,"                                                          a calculated value in the form:");
	printwrap::print_wrap(5,53,"                                                             */<number>");
	printwrap::print_wrap(5,53,"                                                          a combination of these. ");
	printwrap::print_wrap(5,53,"                                                          examples:");
	printwrap::print_wrap(5,71,"                                                              *        (all values)");
	printwrap::print_wrap(5,71,"                                                              1        (one)");
	printwrap::print_wrap(5,71,"                                                              1,5,10   (one, five, and ten)");
	printwrap::print_wrap(5,71,"                                                              1-10     (one through ten, inclusive)");
	printwrap::print_wrap(5,71,"                                                              1,3-6,24 (one, three through six, and twenty-four)");
	printwrap::print_wrap(5,71,"                                                              */15     (any value evenly devisible by 15)");
	printwrap::print_wrap(5,71,"                                                              1,*/20   (one and any value evenly  devisible by twenty)");
	printwrap::print_wrap(5,53,"                                                     For all fields, values outside the indicated range are silently ignored. Well, a job scheduled to happen every month 14 will, in fact, happen on the 14th month.... whenever that finally occurs.");
	printwrap::print_wrap(5,53,"                                                     Ranges for each field are:");
	printwrap::print_wrap(5,71,"                                                         minute        minutes of the hour. 0 to 59");
	printwrap::print_wrap(5,71,"                                                         hour          hours of the day. 0 to 23");
	printwrap::print_wrap(5,71,"                                                         day           days of the month 1 to 28, 29, 30 or 31 (depending on the month/year)");
	printwrap::print_wrap(5,71,"                                                         day_of_week   days of the week 1 to 7 starting with Sunday.");
	printwrap::print_wrap(5,71,"                                                         day_of_year   days of the year 1 to 366");
	printwrap::print_wrap(5,71,"                                                         month         months of the year 1 to  12 starting with January");
	printwrap::print_wrap(5,53,"                                                     A Given job will happen when ALL of the fields match ( an \"and\" condition). For example, a When that specified day_of_week=1 and day_of_month=1 will only occur when the 1st of the month is also the 1st day of the week (a Sunday). In 2025, this would happen only on June 1st.");
	printwrap::print_wrap(5,53,"        [");
	printwrap::print_wrap(5,53,"            {");
	printwrap::print_wrap(5,53,"                \"field\": \"minute\",                   when the minute is");
	printwrap::print_wrap(5,53,"                \"value\": \"15,30,45\"                  15, 30, or 45");
	printwrap::print_wrap(5,53,"            },");
	printwrap::print_wrap(5,53,"            {");
	printwrap::print_wrap(5,53,"                \"field\":\"hour\",                      and the hour is");
	printwrap::print_wrap(5,53,"                \"value\":\"*\"                          every hour (asterisk means anything)");
	printwrap::print_wrap(5,53,"            },");
	printwrap::print_wrap(5,53,"            {");
	printwrap::print_wrap(5,53,"                \"field\":\"day\",                       and the day of the month is");
	printwrap::print_wrap(5,53,"                \"value\":\"*\"                          any day (of the month)");
	printwrap::print_wrap(5,53,"            },");
	printwrap::print_wrap(5,53,"            {");
	printwrap::print_wrap(5,53,"                \"field\":\"day_of_week\",               and the day of the week is");
	printwrap::print_wrap(5,53,"                \"value\":\"1\"                          Sunday (day 1)");
	printwrap::print_wrap(5,53,"            },");
	printwrap::print_wrap(5,53,"            {");
	printwrap::print_wrap(5,53,"                \"field\":\"day_of_year\",               and the day of the year is");
	printwrap::print_wrap(5,53,"                \"value\":\"*\"                          any day (of the year)");
	printwrap::print_wrap(5,53,"            },");
	printwrap::print_wrap(5,53,"            {");
	printwrap::print_wrap(5,53,"                \"field\":\"month\",                     and the month is");
	printwrap::print_wrap(5,53,"                \"value\":\"*\"                          every month");
	printwrap::print_wrap(5,53,"            }");
	printwrap::print_wrap(5,53,"        ],");
	printwrap::print_wrap(5,53,"        \"purge_rule\": ");
	printwrap::print_wrap(5,53,"        {                                            Purge Rule indicates when older snapshots should be destroyed (deleted).");
	printwrap::print_wrap(5,53,"                                                     Purge Rule consists of two properties:");
	printwrap::print_wrap(5,53,"                                                         unit");
	printwrap::print_wrap(5,53,"                                                         value ");
	printwrap::print_wrap(5,53,"                                                     Unit is the type of calculation/period the purge rule will consider.");
	printwrap::print_wrap(5,53,"                                                         Unit can be one of:");
	printwrap::print_wrap(5,70,"                                                             count    purges snapshots when the number (matching the job) exceeds the value of, uh, value. Value is the number of snapshots to retain. The job will delete snapshots when the total number of snapshots for this job exceeds the indicated number. Snapshots will be deleted oldest-first based on the snapshot's creation time.");
	printwrap::print_wrap(5,70,"                                                             minute/hour/day/week/year");
	printwrap::print_wrap(5,70,"                                                                      purges snapshots older than the number of <unit> periods of time specified in the value field.");
	printwrap::print_wrap(5,70,"                                                         Value must be an integer. For example a unit=day and value=10 will delete all this job's snapshots older than 10 days. unit=year and value=2 will delete snapshots older than 2 years. A snapshot's creation time is used for the purpose of determing age and *not* the snapshot's name.");
	printwrap::print_wrap(5,70,"                                                         none         No snapshot purging is done. All snapshots made (for this job) will be kept. The value field must still be present, but is ignored for this case.");
	printwrap::print_wrap(5,70,"                                                     Any job can only have ONE purge rule.");
	printwrap::print_wrap(5,53,"            \"unit\":\"count\",                          Keep a specific number of snapshots");
	printwrap::print_wrap(5,53,"            \"value\":\"72\"                             Where 72 is that number. Once the number of snapshots exceeds the number, older snapshots will be destroyed.");
	printwrap::print_wrap(5,53,"        }");
	printwrap::print_wrap(5,53,"    }");
	printwrap::print_wrap(5,53,"    ,{.... }                                         additional jobs, in the same format.");
	printwrap::print_wrap(5,53,"    ]");
	printwrap::print_wrap(5,53,"}");
	printwrap::print_wrap(5,0,"<end of the config file>");

	printwrap::print_wrap(5,0,"Unicode numbers: https://www.unicode.org/reports/tr44");
	printwrap::print_wrap(5,0,"chrono::format https://docs.rs/chrono/latest/chrono/format/strftime/index.html");

	process::exit(1);
}

fn print_generic_config()
{
	let root = get_root();
	let home = get_home();
	let root_slashes = root.chars().filter(|c| *c == '/').count();
	let home_slashes = home.chars().filter(|c| *c == '/').count();
	if root_slashes < 1
	{
		println!("Root file system \"{}\" doesn't look like a real zfs filesystem path (it's not at least two part separated by a slash \"pool/path\").", root);
		process::exit(4);
	}
	if home_slashes < 1
	{
		println!("Home file system \"{}\" doesn't look like a real zfs filesystem path (it's not at least two part separated by a slash \"pool/path\").", home);
		process::exit(4);
	}

	println!("{{\n\t\"comment\":[\"Znappr configuration.\"],\n\t\"jobs\": [\n\t\t{{\n\t\t\t\"comment\": [\"HOME Take snapshots at 15, 30, and 45 minutes past the hour, retain 36 (approx. 12 hours)\"],\n\t\t\t\"dataset\": \"{}\",\n\t\t\t\"recursive\":true,\n\t\t\t\"prefix\": \"QUARTER__\",\n\t\t\t\"postfix\": \"\",\n\t\t\t\"pre_date\": false,\n\t\t\t\"date_format\":\"%Y-%m-%d-%H:%M\",\n\t\t\t\"whens\":\n\t\t\t[\n\t\t\t\t{{\n\t\t\t\t\t\"field\": \"minute\",\n\t\t\t\t\t\"value\": \"15,30,45\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"hour\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_week\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_year\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"month\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}}\n\t\t\t],\n\t\t\t\"purge_rule\": {{\"unit\":\"count\", \"value\":\"36\"}}\n\t\t}},\n\t\t{{\n\t\t\t\"comment\": [\"HOME Take snapshots hourly, retain 48 (approx. 2 days)\"],\n\t\t\t\"dataset\": \"{}\",\n\t\t\t\"recursive\":true,\n\t\t\t\"prefix\": \"HOUR_____\",\n\t\t\t\"postfix\": \"\",\n\t\t\t\"pre_date\": false,\n\t\t\t\"date_format\":\"%Y-%m-%d-%H:00\",\n\t\t\t\"whens\":\n\t\t\t[\n\t\t\t\t{{\n\t\t\t\t\t\"field\": \"minute\",\n\t\t\t\t\t\"value\": \"0\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"hour\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_week\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_year\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"month\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}}\n\t\t\t],\n\t\t\t\"purge_rule\": {{\"unit\":\"count\", \"value\":\"48\"}}\n\t\t}},\n\t\t{{\n\t\t\t\"comment\": [\"HOME Take snapshots daily, retain 8 (approx. 1 week)\"],\n\t\t\t\"dataset\": \"{}\",\n\t\t\t\"recursive\":true,\n\t\t\t\"prefix\": \"DAY______\",\n\t\t\t\"postfix\": \"\",\n\t\t\t\"pre_date\": true,\n\t\t\t\"date_format\":\"%Y-%m-%d\",\n\t\t\t\"whens\":\n\t\t\t[\n\t\t\t\t{{\n\t\t\t\t\t\"field\": \"minute\",\n\t\t\t\t\t\"value\": \"0\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"hour\",\n\t\t\t\t\t\"value\":\"0\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_week\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_year\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"month\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}}\n\t\t\t],\n\t\t\t\"purge_rule\": {{\"unit\":\"count\", \"value\":\"8\"}}\n\t\t}},\n\t\t{{\n\t\t\t\"comment\": [\"ROOT Take snapshots daily, retain 8 (approx. 1 week)\"],\n\t\t\t\"dataset\": \"{}\",\n\t\t\t\"recursive\":false,\n\t\t\t\"prefix\": \"DAY______\",\n\t\t\t\"postfix\": \"\",\n\t\t\t\"pre_date\": true,\n\t\t\t\"date_format\":\"%Y-%m-%d\",\n\t\t\t\"whens\":\n\t\t\t[\n\t\t\t\t{{\n\t\t\t\t\t\"field\": \"minute\",\n\t\t\t\t\t\"value\": \"0\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"hour\",\n\t\t\t\t\t\"value\":\"0\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_week\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"day_of_year\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}},\n\t\t\t\t{{\n\t\t\t\t\t\"field\":\"month\",\n\t\t\t\t\t\"value\":\"*\"\n\t\t\t\t}}\n\t\t\t],\n\t\t\t\"purge_rule\": {{\"unit\":\"count\", \"value\":\"8\"}}\n\t\t}}\n\t]\n}}\n", home,home,home,root);

	process::exit(1);
}

fn load_config(file_path: &str) -> Znappr
{
	debug!("Json_file_path: \"{}\"", file_path);


	let json_file_path = Path::new(file_path);
	let file = File::open(json_file_path).expect("file not found");
	let znappr=serde_json::from_reader(file).expect("error while reading");
	return znappr;
}

fn adfix_is_valid(test:&str) -> bool
{
	let mut is_valid:bool = true;
	for c in test.chars()
	{
		let c_is_valid = c.is_alphanumeric() || c.is_whitespace() || ((c == '.')||(c == '_')||(c == '-'));
		is_valid = is_valid && c_is_valid;
	} 
	return is_valid
}
fn walk(znappr:&Znappr)
{
	println!("Walking znappr...");
	for i in &znappr.jobs 
	{
		println!("Comment:");
		for n in &i.comment
		{
			println!("\t\"{}\"", n);
		}
		println!("Prefix:\"{}\"",i.prefix);
		if !adfix_is_valid(i.prefix.as_str())
		{
			println!("Prefix is not valid! No snapshot for this job will ever be made.");
			process::exit(3);
		}
		println!("Postfix:\"{}\"",i.postfix);
		if !adfix_is_valid(i.postfix.as_str())
		{
			println!("Postfix is not valid! No snapshot for this job will ever be made.");
			process::exit(3);
		}
		println!("Date_format:\"{}\"",i.date_format);
		for w in &i.whens
		{
			match w.field.as_str()
			{
				"minute"|"hour"|"day"|"day_of_week"|"day_of_year"|"month" =>
				{
					println!("\t\t\"{}\":\"{}\"", w.field,w.value);
				}
				_ =>
				{
					println!("\tUnknown match field \"{}\". This job will always fail.",w.field);
					process::exit(4);
				}
			}
			if ! validate_when_value(w.value.as_str())
			{
				println!("When range is invalid: \"{}\"", w.value);
				process::exit(5);
			}
		}
	}
	println!("Done walking config.");
	process::exit(2);
}

fn validate_when_value(value:&str) -> bool
{
	let mut valid=false;
	let parts = value.split(",");
	//debug!("\t\tValue: \"{}\" time:{}",value, time_part);
	for p in parts
	{
		//debug!("\t\tPart:\"{}\"", p);
		if p == "*"
		{
			valid=true;
			break;
		}
		else
		{
			let first_last = p.split("-");
			let range = first_last.collect::<Vec<&str>>();
			let math = p.split("/");
			let num_denom = math.collect::<Vec<&str>>();
			if range.len() == 2
			{
				//range
				// check if number is a range, is "%d-%d", 
				let first:u32 = range[0].parse().unwrap();
				let last:u32 = range[1].parse().unwrap();
				if (first <= 1000) && (last <= 1000)
				{
					valid=true;
					break;
				}
			}
			else if num_denom.len() == 2
			{
				let denominator:u32 = num_denom[1].parse().unwrap();
				if (denominator > 0) && (denominator <= 1000)
				{
					valid=true;
					break;
				}
			}
			else
			{
				// single value
				let test_number: u32 = p.parse().unwrap();
				if test_number <= 1000
				{
					valid=true;
					break;
				}
			}
		}
	}
	return valid;
}

fn check_values(value:&str, time_part:u32) -> bool
{
	let mut matched=false;
	let parts = value.split(",");
	debug!("\t\tValue: \"{}\" time:{}",value, time_part);
	for p in parts
	{
		debug!("\t\tPart:\"{}\"", p);
		if p == "*"
		{
			debug!("\t\tPart is *. Automatic Match");
			matched=true;
			break;
		}
		else
		{
			let first_last = p.split("-");
			let range = first_last.collect::<Vec<&str>>();
			let math = p.split("/");
			let num_denom = math.collect::<Vec<&str>>();
			if range.len() == 2
			{
				//range
				// check if number is a range, is "%d-%d", 
				let first:u32 = range[0].parse().unwrap();
				let last:u32 = range[1].parse().unwrap();
				if (time_part >= first) && (time_part <= last)
				{
					matched=true;
					break;
				}
			}
			else if num_denom.len() == 2
			{
				let denominator:u32 = num_denom[1].parse().unwrap();
				if (time_part % denominator) == 0
				{
					matched=true;
					break;
				}
			}
			else
			{
				// single value
				let test_number: u32 = p.parse().unwrap();
				if test_number == time_part
				{
					matched=true;
					break;
				}
			}
		}
	}
	return matched;
}
fn delete_snapshot(snapshot:&String, recursive:bool) -> bool
{
	let mut success=false;
	info!("Delete {}", snapshot);
	let full_command = format!("zfs destroy {}",snapshot);
	debug!("{}",full_command);
	let result_of_snapshot_delete =	if recursive
		{
			Command::new("zfs")
				.arg("destroy")
				.arg("-r")
				.arg(snapshot)
				.output()
				.expect("failed to execute process")
		}
		else
		{
			Command::new("zfs")
				.arg("destroy")
				.arg(snapshot)
				.output()
				.expect("failed to execute process")
		};
	let return_code = result_of_snapshot_delete.status;
	let stdout = String::from_utf8(result_of_snapshot_delete.stdout).unwrap();

	trace!("{}", stdout);
	if !return_code.success()
	{
		error!("Error deleting snapshot:");
		error!("stderr: {}", String::from_utf8_lossy(&result_of_snapshot_delete.stderr))
	}
	else
	{
		success=true;
		info!("Success deleting snapshot.");
	}
	return success
}
//zfs list -t filesystem -o name,mountpoint
fn get_root() -> String
{
	return get_filesystem(" \\/$")
}
fn get_home() -> String
{
	return get_filesystem(" \\/home$")
}
fn get_filesystem(pattern:&str) -> String
{
	debug!("zfs list -t filesystem -o name,mountpoint");
	let filesystem_list = Command::new("zfs")
			.arg("list")
			.arg("-t")
			.arg("filesystem")
			.arg("-o")
			.arg("name,mountpoint")
			.arg("-s")
			.arg("createtxg") // orders by createtxg, oldest first so clones won't be selected.
			.stdout(Stdio::piped())
			.spawn()
			.expect("failed to execute process");
	let filesystem_list_out = filesystem_list.stdout.expect("Failed to open zfs list stdout");
	let filesystem_grep = Command::new("grep")
			.arg(pattern)
			.stdin(Stdio::from(filesystem_list_out))
			.stdout(Stdio::piped())
			.output()
			.unwrap();

	let stdout = String::from_utf8(filesystem_grep.stdout).unwrap();
	let mut parts = stdout.split_whitespace();
	return String::from(parts.next().unwrap());
}

fn get_snapshots(dataset: &String, prefix:&String, postfix:&String) -> Vec<Snapshot>
{
	let full_command = format!("zfs list -t snapshot -H | grep '{}@{}.*{}'| sort| awk '{{print $1}}'",dataset,prefix,postfix);
	debug!("{}",full_command);
	let snapshot_list = Command::new("zfs")
			.arg("list")
			.arg("-t")
			.arg("snapshot")
			.arg("-H")
			.arg("-o")
			.arg("name,creation")
			.arg("-p")	// creation date will be unix epoch (seconds)
			.arg("-S")	// need to sort backwards as we will always delete oldest-first
			.arg("creation")
			.arg(dataset)
			.stdout(Stdio::piped())
			.spawn()
			.expect("failed to execute process");
	let snapshot_list_out = snapshot_list.stdout.expect("Failed to open zfs list stdout");
	let grep_pattern = format!("{}@{}.*{}",dataset,prefix,postfix);
	let snapshot_grep = Command::new("grep")
			.arg(grep_pattern)
			.stdin(Stdio::from(snapshot_list_out))
			.stdout(Stdio::piped())
			.output()
			.unwrap();

	let stdout = String::from_utf8(snapshot_grep.stdout).unwrap();
	let lines = stdout.lines();
	let mut snapshots: Vec<Snapshot> = Vec::new();
	for l in lines
	{
		trace!("Snapshot:\"{}\"", l);
		let lparts = l.split("\t");
		let parts = lparts.collect::<Vec<&str>>();
		let snap = Snapshot{snapshot:String::from(parts[0]),date:String::from(parts[1])};
		trace!("\t{}\t{}", snap.snapshot, snap.date);
		snapshots.push(snap);
	}
	debug!("Snapshots lines returned: {}", snapshots.len());
	return snapshots
}

fn purge_snapshots(dataset: &String, recursive:bool, prefix:&String, postfix:&String, purge_rule:&String, purge_value:&String, systemtime:i64)
{
	info!("\tPurge {}", dataset);
	let mut snapshots_deleted = 0;
	let mut snapshots = get_snapshots(dataset, prefix, postfix);
	let snapshot_count:u32 = snapshots.len() as u32; // snapshot len cannot plausibly exceed u32.
	match purge_rule.as_str()
	{
		"count" =>
			{
				info!("Purge rule \"count\", keep {}", purge_value);
				let count:u32 = purge_value.parse().unwrap();
				if count < snapshot_count
				{
					let delete_count = snapshot_count - count;
					info!("Delete {} snapshots out of {}.", delete_count, snapshot_count);
					for d in 1..=delete_count
					{
						//let snapshot_to_delete = vlines.pop().unwrap();
						let snapshot_to_delete:Snapshot = snapshots.pop().unwrap();
						info!("Deleting snapshot {} of {}:\"{}\"", d, delete_count, snapshot_to_delete.snapshot);
						delete_snapshot(&snapshot_to_delete.snapshot, recursive);
					}
				}
			},
		//minute/hour/day/week/year
		"minute"|"hour"|"day"|"week"|"year" =>
			{
				let count:u32 = purge_value.parse().unwrap();
				let seconds:u32;
				info!("Purge rule keep for {} {}", count, purge_rule);
				match purge_rule.as_str()
				{
					"minute"=> seconds = count * 60,
					"hour" =>  seconds = count * 60 * 60,
					"day" =>   seconds = count * 60 * 60 * 24,
					"week" =>  seconds = count * 60 * 60 * 24 * 7,
					"year" =>  seconds = count * 60 * 60 * 24 * 365,
					_      =>  seconds = count * 60 * 60 * 24 * 365,
				}
				info!("Purge snapshots more than {} seconds old.", seconds);
				debug!("Current time {}", systemtime);
				for snap in snapshots
				{
					let snaptime:i64 = snap.date.parse().unwrap();
					let offset = systemtime-snaptime;
					let purge: bool = {
						if offset > seconds as i64 
						{
							true 
						}
						else
						{
							false
						}
					};
					trace!("Snapshot {} from {} (offset:{}, purge: {})", snap.snapshot, snap.date, offset, purge);
					if purge
					{
						info!("Deleting snapshot:\"{}\"", snap.snapshot);
						if delete_snapshot(&snap.snapshot, recursive)
						{
							snapshots_deleted = snapshots_deleted +1;
						}
					}
				}
				//purge_snapshots_older_than
			},
		_ =>
			{
				//all
			},
	}
	info!("Snapshots deleted: {}", snapshots_deleted);
}

fn take_snapshot(dataset:&String, recursive:bool, prefix:&String, postfix:&String, date_format:&String, pre_date:bool,
				year:i32, yearx:i32, month:u32, monthx:u32, day:u32, dayx:u32, hour:u32, minute:u32)
{
	debug!("RUN JOB:");
	let mut date_time = Local.with_ymd_and_hms(year, month, day, hour, minute, 0).unwrap();
	if pre_date
	{
		date_time = Local.with_ymd_and_hms(yearx, monthx, dayx, 0, 0, 0).unwrap();
	}

	let formatted_date = format!("{}", date_time.format(date_format));
	let tag = format!("{}{}{}",prefix, formatted_date, postfix);
	if tag.len() <= 0
	{
		error!("Snapshot tag is an empty string \"\". Can't take that snapshot. Change the prefix, postfix, or date fields for this job to something not-empty.");

	}
	else
	{
		let snapshot_label = format!("{}@{}", dataset, tag);
		let command_line = format!("zfs snapshot \"{}\"", snapshot_label);
		info!("Take Snapshot \"{}\"", snapshot_label);
		debug!("\t\tSnapshot Command Line:\"{}\"",command_line);
		let result_of_snapshot = if recursive
			{
				Command::new("zfs")
					.arg("snapshot")
					.arg("-r")
					.arg(snapshot_label)
					.output()
					.expect("failed to execute process")
			}
			else
			{
				Command::new("zfs")
					.arg("snapshot")
					.arg(snapshot_label)
					.output()
					.expect("failed to execute process")
			};
		let return_code = result_of_snapshot.status;
		if !return_code.success()
		{
			error!("Error making snapshot:");
			error!("stderr: {}", String::from_utf8_lossy(&result_of_snapshot.stderr))
		}
		else
		{
			info!("Snapshot taken!");
		}
	}
}

fn process_jobs(znappr:&Znappr, today:DateTime<Local>)
{
	//let today = Local::now(); 
	let yesterday = today - Duration::days(1);
	//trace!("Local time now is {}", local_time);
	let year:i32 = today.year();
	let yearx:i32 = yesterday.year();
	let month:u32 = today.month();
	let monthx:u32 = yesterday.month();
	let day:u32 = today.day();
	let dayx:u32 = yesterday.day();
	let dayofyear:u32=today.ordinal();
	let dayofweekwd:Weekday=today.weekday();
	let dayofweek = dayofweekwd.number_from_sunday();
	let hour:u32 = today.hour();
	let minute:u32 = today.minute();
	let seconds:i64 = today.timestamp();
	let mut jobs_run:i32 = 0;

	info!("Today : {}-{}-{}:{}:{} ({})",year,month,day,hour,minute, seconds);
	info!("Yesterday : {}-{}-{}",yearx,monthx,dayx);

'jobs:	for j in &znappr.jobs 
	{
		let mut runjob=true;
		info!("--------------------------");
		info!("JOB");
		for c in &j.comment
		{
			info!("\"{}\"", c);
		}	
		info!("\tDataset    :\"{}\"", j.dataset);
		info!("\t  Recursive:\"{}\"", j.recursive);
		info!("\tPrefix     :\"{}\"", j.prefix);
		info!("\tPostfix    :\"{}\"", j.postfix);
		info!("\tDate Format:\"{}\"", j.date_format);
		if !adfix_is_valid(j.prefix.as_str())
		{
			error!("\tPREFIX is invalid. Skipping job.");
			continue 'jobs;
		}
		if !adfix_is_valid(j.postfix.as_str())
		{
			error!("\tPOSTFIX is invalid. Skipping job.");
			continue 'jobs;
		}
		for w in &j.whens
		{
			let mut condition=false;
			let field:&str= &w.field;
			match field
			{
				"minute" =>
				{
					info!("\t\tMINUTE:\"{}\"",w.value);
					condition = check_values(&w.value,minute);
				}
				"hour" =>
				{
					info!("\t\tHOUR \"{}\"", w.value);
					condition = check_values(&w.value,hour);
				}
				"day" =>
				{
					info!("\t\tDAY \"{}\"", w.value);
					condition = check_values(&w.value,day);
				}
				"day_of_week" =>
				{
					info!("\t\tDAY_OF_WEEK \"{}\"", w.value);
					condition = check_values(&w.value,dayofweek);
				}
				"day_of_year" =>
				{
					info!("\t\tDAY_OF_YEAR \"{}\"", w.value);
					condition = check_values(&w.value,dayofyear);
				}
				"month" =>
				{
					info!("\t\tMONTH \"{}\"", w.value);
					condition = check_values(&w.value,month);
				}
				_ =>
				{
					error!("\tUnknown time field \"{}\" will fail automatically.",w.field);
				}
			}
			runjob = runjob & condition;
			if condition
			{
				info!("\t\tMATCHED");
			}
			else
			{
				info!("\t\tNOT MATCHED");
			}
		}
		if runjob
		{
			jobs_run = jobs_run +1;
			take_snapshot(&j.dataset, j.recursive, &j.prefix, &j.postfix, &j.date_format, j.pre_date,
				year, yearx, month, monthx, day, dayx, hour, minute);
			// purging happens ONLY if the job ran
			purge_snapshots(&j.dataset, j.recursive, &j.prefix, &j.postfix, &j.purge_rule.unit, &j.purge_rule.value, seconds);
		}
		else
		{
			info!("\tDO NOT RUN JOB!");
		}
	}
	info!("Total jobs run: {}", jobs_run);
}

fn main() 
{
	let args: Vec<String> = env::args().collect();
	let start=1;
	let end=args.len();
	let mut verbose = log::Level::Info; // default log level of INFO
	let mut do_walk=false;
	let mut json_file_path="/local/etc/znappr/znappr.json";
	let mut today=Local::now();
	let mut fakedate:&str = "";
	let mut parse_fake_date=false;
	let mut skip_argument=false;
	for i in start..end
	{
		if skip_argument
		{
			skip_argument = false;
		}
		else
		{
			match args[i].as_ref()
			{
				"-h" | "--help" =>
					{
					usage();
					}
				"-j" =>
					{
						config_format();
					}
				"-p" =>
					{
						print_generic_config();
					}
				"-f" =>
					{
						if (i+1) < end
						{
							json_file_path=&args[i+1];
							skip_argument = true;
						}
					}
				"-v" =>
					{
						verbose = log::Level::Debug;
					} 
				"-vv" =>
					{
						verbose = log::Level::Trace;
					} 
				"-c"|"--configtest" =>
					{
						do_walk=true;
					}
				"--date" =>
					{
						if (i+1) < end
						{
							fakedate=&args[i+1];
							parse_fake_date=true;
							skip_argument=true;
						}
					}
				_ =>
					{
						println!("Unknown argument \"{}\".",args[i]);
						usage();
					}

			}
		}
	}
	stderrlog::new().module(module_path!()).verbosity(verbose).init().unwrap();

	if parse_fake_date
	{
		trace!("Parsing date \"{}\"", fakedate);
		let fake_today = NaiveDateTime::parse_from_str(fakedate, "%Y-%m-%d-%H:%M").unwrap();
		today = Local.from_local_datetime(&fake_today).unwrap();
		trace!("Parsed today (from fake date):{}", today);
	}

	let znappr = load_config(json_file_path);
	if do_walk
	{
		walk(&znappr);
		process::exit(2);
	}
	process_jobs(&znappr, today)
}

