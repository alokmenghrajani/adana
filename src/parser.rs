use crate::prelude::*;
use strum::{EnumString, EnumVariantNames, VariantNames};

#[derive(Debug, EnumString, EnumVariantNames)]
pub enum CacheCommand<'a> {
    Add { aliases: Vec<&'a str>, value: &'a str },
    List,
    CurrentCache,
    ListCache,
    RemoveCache(Option<&'a str>),
    Remove(&'a str),
    Concat(&'a str),
    Get(&'a str),
    Exec { key: &'a str, args: Option<&'a str> },
    Cd(&'a str),
    Using(&'a str),
    Dump(Option<&'a str>),
    Clear,
    Help,
}

impl CacheCommand<'_> {
    pub const fn doc() -> &'static [(&'static [&'static str], &'static str)] {
        const VARIANTS: &[&str] = CacheCommand::VARIANTS;
        assert!(14 == VARIANTS.len(), "enum doc no longer valid!");
        &[
            (&["put"], "Put a new value to current cache. can have multiple aliases with option '-a'. e.g `set -a drc -a drcomp docker-compose`"),
            (&["describe","ds"], "List values within the cache."),
            (&["listcache","lsch"], "List available caches."),
            (&["del","delete"], "Remove value from cache. Accept either a hashkey or an alias. e.g `del drc`"),
            (&["get"], "Get value from cache. Accept either a hashkey or an alias. e.g `get drc`"),
            (&["exec","run"], "Run a value from the cache as an OS command. Accept either a hashkey or an alias. e.g `run drc`"),
            (&["use","using"], "Use another cache context default cache is DEFAULT. e.g `use linux`"),
            (&["dump"], "Dump cache(s) as json. Take an optional parameter, the cache name. e.g `dump linux`"),
            (&["clear","cls"], "Clear the terminal."),
            (&["delch","deletecache"], "Delete cache or clear current cache value."),
            (&["currch","currentcache"], "Current cache."),
            (&["cd"], "Navigate to a directory"),
            (&["merge/mergech"], "Merge current with a given cache"),
            (&["help"], "Display Help."),
        ]
    }
}

fn add_command(command: &str) -> Res<CacheCommand> {
    map(
        pair(
            preceded(
                preceded(multispace0, tag_no_case("PUT")),
                many0(preceded(
                    preceded(multispace1, tag_no_case("-a")),
                    preceded(
                        multispace1,
                        cut(verify(
                            take_while(|c: char| c.is_alphanumeric()),
                            |s: &str| !s.is_empty(),
                        )),
                    ),
                )),
            ),
            preceded(
                multispace1,
                cut(verify(rest.map(|s: &str| s.trim()), |s: &str| {
                    !s.is_empty()
                })),
            ),
        ),
        |(aliases, value)| CacheCommand::Add { aliases, value },
    )(command)
}
fn del_command(command: &str) -> Res<CacheCommand> {
    map(
        alt((
            extract_key(tag_no_case("del")),
            extract_key(tag_no_case("delete")),
        )),
        CacheCommand::Remove,
    )(command)
}
fn get_command(command: &str) -> Res<CacheCommand> {
    map(extract_key(tag_no_case("GET")), CacheCommand::Get)(command)
}
fn concat_command(command: &str) -> Res<CacheCommand> {
    map(
        alt((
            extract_key(tag_no_case("MERGE")),
            extract_key(tag_no_case("MERGECH")),
        )),
        CacheCommand::Concat,
    )(command)
}
fn exec_command(command: &str) -> Res<CacheCommand> {
    map(
        pair(
            alt((
                extract_key(tag_no_case("EXEC")),
                extract_key(tag_no_case("RUN")),
                take_till1(|s: char| s.is_whitespace()),
            )),
            opt(rest.map(|r: &str| r.trim())),
        ),
        |(key, args)| CacheCommand::Exec { key, args },
    )(command)
}
fn cd_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            tag_no_case("CD"),
            preceded(multispace1, rest.map(|r: &str| r.trim())),
        ),
        CacheCommand::Cd,
    )(command)
}

fn list_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            alt((tag_no_case("describe"), tag("ds"))),
            verify(rest, |s: &str| s.trim().is_empty() || s == "\n"),
        ),
        |_| CacheCommand::List,
    )(command)
}
fn current_cache_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            alt((tag_no_case("CURRCH"), tag("currentcache"))),
            cut(verify(rest, |s: &str| s.trim().is_empty() || s == "\n")),
        ),
        |_| CacheCommand::CurrentCache,
    )(command)
}

fn help_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            tag_no_case("HELP"),
            cut(verify(rest, |s: &str| s.trim().is_empty() || s == "\n")),
        ),
        |_| CacheCommand::Help,
    )(command)
}
fn clear_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            alt((tag_no_case("CLS"), tag_no_case("CLEAR"))),
            cut(verify(rest, |s: &str| s.trim().is_empty() || s == "\n")),
        ),
        |_| CacheCommand::Clear,
    )(command)
}
fn list_cache_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            alt((tag_no_case("LISTCACHE"), tag("lsch"))),
            cut(verify(rest, |s: &str| s.trim().is_empty() || s == "\n")),
        ),
        |_| CacheCommand::ListCache,
    )(command)
}

fn extract_key<'a, F>(parser: F) -> impl Fn(&'a str) -> Res<&'a str>
where
    F: Fn(&'a str) -> Res<&'a str>,
{
    move |s: &str| {
        preceded(
            &parser,
            preceded(
                multispace1,
                take_while1(|s: char| {
                    s.is_alphanumeric() || s == '-' || s == '_'
                }),
            ),
        )(s)
    }
}

fn using_command(command: &str) -> Res<CacheCommand> {
    map(
        alt((
            extract_key(tag_no_case("USING")),
            extract_key(tag_no_case("USE")),
        )),
        CacheCommand::Using,
    )(command)
}

fn dump_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            tag_no_case("DUMP"),
            cut(verify(rest, |s: &str| {
                s.is_empty() || s.starts_with(' ') || s == "\n"
            }))
            .and_then(opt(preceded(
                multispace1,
                take_while1(|s: char| s.is_alphanumeric() || s == '-'),
            ))),
        ),
        CacheCommand::Dump,
    )(command)
}
fn del_cache_command(command: &str) -> Res<CacheCommand> {
    map(
        preceded(
            alt((tag_no_case("DELCH"), tag_no_case("deletecache"))),
            cut(verify(rest, |s: &str| {
                s.is_empty() || s.starts_with(' ') || s == "\n"
            }))
            .and_then(opt(preceded(multispace1, rest.map(|s: &str| s.trim())))),
        ),
        CacheCommand::RemoveCache,
    )(command)
}

pub fn parse_command(command: &str) -> Res<CacheCommand> {
    preceded(
        multispace0,
        alt((
            add_command,
            cd_command,
            del_command,
            get_command,
            using_command,
            dump_command,
            list_cache_command,
            concat_command,
            current_cache_command,
            del_cache_command,
            list_command,
            help_command,
            clear_command,
            exec_command,
        )),
    )(command)
}
