//! Every API resource and operation the CLI supports. Option validation,
//! request building, and output formatting all read from this table.

pub struct Resource {
    pub name: &'static str,
    /// Used in messages such as "issue ID required".
    pub singular: &'static str,
    /// Nested under `/projects/<--project>/`.
    pub project_scoped: bool,
    pub operations: &'static [Operation],
}

pub struct Operation {
    pub name: &'static str,
    pub method: Method,
    /// Appended to the resource path; `{id}` is the second positional argument.
    pub path: &'static str,
    /// Operations with a query accept `--query` plus the typed query fields.
    pub query: Option<Query>,
    pub body: Body,
    pub expected: &'static [u16],
    pub view: View,
    pub requires_api_key: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

#[derive(Clone, Copy)]
pub enum Kind {
    Text,
    Bool,
}

/// A typed flag that maps to a query parameter or JSON body key.
pub struct Field {
    pub flag: &'static str,
    pub key: &'static str,
    pub kind: Kind,
}

pub struct Query {
    pub fields: &'static [Field],
    /// At least one of these flags must be given unless `--query` is.
    pub require_any: &'static [&'static str],
}

pub enum Body {
    None,
    /// `--body` may be sent.
    OptionalRaw,
    /// `--body` must be sent.
    Raw,
    /// Built from typed flags unless `--body` is given.
    Typed(Typed),
}

pub struct Typed {
    pub fields: &'static [Field],
    /// Flags that must be given, each with a hint appended to the error.
    pub required: &'static [(&'static str, &'static str)],
    /// Body keys filled in when their flag is absent.
    pub defaults: &'static [(&'static str, &'static str)],
    /// At least one of these flags must be given.
    pub require_any: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Record {
    Project,
    Issue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// Print the response body unchanged.
    Raw,
    Table(Record),
    Detail(Record),
    Created(Record),
    Updated(Record),
    Deleted(Record),
    ErrorReport,
}

impl Resource {
    pub fn operation(&self, name: &str) -> Option<&Operation> {
        self.operations.iter().find(|op| op.name == name)
    }
}

impl Operation {
    pub fn takes_id(&self) -> bool {
        self.path.contains("{id}")
    }
    /// Whether `flag` applies to this operation of `resource`.
    pub fn accepts(&self, resource: &Resource, flag: &str) -> bool {
        let in_fields = |fields: &[Field]| fields.iter().any(|field| field.flag == flag);
        (resource.project_scoped && flag == "--project")
            || self
                .query
                .as_ref()
                .is_some_and(|query| flag == "--query" || in_fields(query.fields))
            || match &self.body {
                Body::None => false,
                Body::OptionalRaw | Body::Raw => flag == "--body",
                Body::Typed(typed) => flag == "--body" || in_fields(typed.fields),
            }
    }
}

pub fn find(name: &str) -> Option<&'static Resource> {
    RESOURCES.iter().find(|resource| resource.name == name)
}

const fn op(name: &'static str, method: Method, path: &'static str) -> Operation {
    Operation {
        name,
        method,
        path,
        query: None,
        body: Body::None,
        expected: &[200],
        view: View::Raw,
        requires_api_key: true,
    }
}
impl Operation {
    const fn query(mut self, fields: &'static [Field]) -> Self {
        self.query = Some(Query {
            fields,
            require_any: &[],
        });
        self
    }
    const fn search(
        mut self,
        fields: &'static [Field],
        require_any: &'static [&'static str],
    ) -> Self {
        self.query = Some(Query {
            fields,
            require_any,
        });
        self
    }
    const fn body(mut self, body: Body) -> Self {
        self.body = body;
        self
    }
    const fn expect(mut self, expected: &'static [u16]) -> Self {
        self.expected = expected;
        self
    }
    const fn view(mut self, view: View) -> Self {
        self.view = view;
        self
    }
    /// Authenticated by a token in the URL rather than an API key.
    const fn token_auth(mut self) -> Self {
        self.requires_api_key = false;
        self
    }
}
const fn text(flag: &'static str, key: &'static str) -> Field {
    Field {
        flag,
        key,
        kind: Kind::Text,
    }
}
const fn boolean(flag: &'static str, key: &'static str) -> Field {
    Field {
        flag,
        key,
        kind: Kind::Bool,
    }
}

use Method::{Delete, Get, Patch, Post};

const ID: &str = "/{id}";
const CREATED: &[u16] = &[201];
const CREATED_OR_OK: &[u16] = &[200, 201];
const NO_CONTENT_OR_OK: &[u16] = &[204, 200];

const PROJECT_FIELDS: &[Field] = &[
    text("--name", "name"),
    text("--prefix", "prefix"),
    text("--description", "description"),
];
const ISSUE_UPDATE_FIELDS: &[Field] = &[
    text("--title", "title"),
    text("--description", "description"),
    text("--type", "type"),
    text("--status", "status"),
    text("--priority", "priority"),
    text("--email", "submitter_email"),
];
const ISSUE_CREATE_FIELDS: &[Field] = &[
    text("--project", "project_id"),
    text("--title", "title"),
    text("--description", "description"),
    text("--type", "type"),
    text("--status", "status"),
    text("--priority", "priority"),
    text("--email", "submitter_email"),
];

/// Read, write, and delete operations shared by most resources.
const fn crud(record: Option<Record>) -> [Operation; 5] {
    let (table, detail, created, updated, deleted) = match record {
        Some(r) => (
            View::Table(r),
            View::Detail(r),
            View::Created(r),
            View::Updated(r),
            View::Deleted(r),
        ),
        None => (View::Raw, View::Raw, View::Raw, View::Raw, View::Raw),
    };
    [
        op("list", Get, "").query(&[]).view(table),
        op("get", Get, ID).view(detail),
        op("create", Post, "")
            .body(Body::Raw)
            .expect(CREATED)
            .view(created),
        op("update", Patch, ID).body(Body::Raw).view(updated),
        op("delete", Delete, ID)
            .expect(NO_CONTENT_OR_OK)
            .view(deleted),
    ]
}

const PROJECTS: [Operation; 5] = {
    let [list, get, create, update, delete] = crud(Some(Record::Project));
    [
        list,
        get,
        create.body(Body::Typed(Typed {
            fields: PROJECT_FIELDS,
            required: &[
                ("--name", ""),
                (
                    "--prefix",
                    " The API requires a project prefix of 1-10 uppercase letters or numbers.",
                ),
            ],
            defaults: &[],
            require_any: &[],
        })),
        update.body(Body::Typed(Typed {
            fields: PROJECT_FIELDS,
            required: &[],
            defaults: &[],
            require_any: &["--name", "--description", "--prefix"],
        })),
        delete,
    ]
};

const ISSUES: [Operation; 5] = {
    let [list, get, create, update, delete] = crud(Some(Record::Issue));
    [
        list.query(&[
            text("--project", "project_id"),
            text("--status", "status"),
            text("--type", "type"),
            text("--page", "page"),
            text("--per-page", "per_page"),
        ]),
        get,
        create.body(Body::Typed(Typed {
            fields: ISSUE_CREATE_FIELDS,
            required: &[("--project", ""), ("--title", ""), ("--type", "")],
            defaults: &[("priority", "medium"), ("status", "new")],
            require_any: &[],
        })),
        update.body(Body::Typed(Typed {
            fields: ISSUE_UPDATE_FIELDS,
            required: &[],
            defaults: &[],
            require_any: &[
                "--title",
                "--description",
                "--type",
                "--status",
                "--priority",
                "--email",
            ],
        })),
        delete,
    ]
};

const ERRORS: [Operation; 6] = [
    op("list", Get, "").query(&[
        text("--status", "status"),
        boolean("--muted", "muted"),
        text("--page", "page"),
        text("--per-page", "per_page"),
    ]),
    op("get", Get, ID).view(View::ErrorReport),
    op("search", Get, "/search").search(
        &[
            text("--module", "module"),
            text("--function", "function"),
            text("--file", "file"),
            text("--page", "page"),
            text("--per-page", "per_page"),
        ],
        &["--module", "--function", "--file"],
    ),
    op("report", Post, "").body(Body::Raw).expect(CREATED_OR_OK),
    op("create", Post, "").body(Body::Raw).expect(CREATED_OR_OK),
    op("update", Patch, ID).body(Body::Typed(Typed {
        fields: &[text("--status", "status"), boolean("--muted", "muted")],
        required: &[],
        defaults: &[],
        require_any: &["--status", "--muted"],
    })),
];

const INCIDENTS: [Operation; 6] = [
    op("list", Get, "").query(&[]),
    op("get", Get, ID),
    op("report", Post, "").body(Body::Raw).expect(CREATED_OR_OK),
    op("create", Post, "").body(Body::Raw).expect(CREATED_OR_OK),
    op("update", Patch, ID).body(Body::Raw),
    op("resolve", Post, "/{id}/resolve"),
];

const CHECKS: [Operation; 6] = {
    let [list, get, create, update, delete] = crud(None);
    [
        list,
        get,
        create,
        update,
        delete,
        op("results", Get, "/{id}/results").query(&[]),
    ]
};

const HEARTBEATS: [Operation; 9] = {
    let [list, get, create, update, delete] = crud(None);
    [
        list,
        get,
        create,
        update,
        delete,
        op("pings", Get, "/{id}/pings").query(&[]),
        op("ping", Post, "/{id}/ping")
            .body(Body::OptionalRaw)
            .expect(NO_CONTENT_OR_OK)
            .token_auth(),
        op("fail", Post, "/{id}/ping/fail")
            .body(Body::OptionalRaw)
            .expect(NO_CONTENT_OR_OK)
            .token_auth(),
        op("start", Post, "/{id}/ping/start")
            .body(Body::OptionalRaw)
            .expect(NO_CONTENT_OR_OK)
            .token_auth(),
    ]
};

const CLOUD_IP_RANGES: [Operation; 2] = [
    op("list", Get, "").query(&[
        text("--snapshot-id", "snapshot_id"),
        text("--page", "page"),
        text("--per-page", "per_page"),
    ]),
    op("sync-state", Get, "/sync-state"),
];

pub const RESOURCES: &[Resource] = &[
    Resource {
        name: "projects",
        singular: "project",
        project_scoped: false,
        operations: &PROJECTS,
    },
    Resource {
        name: "issues",
        singular: "issue",
        project_scoped: false,
        operations: &ISSUES,
    },
    Resource {
        name: "errors",
        singular: "error",
        project_scoped: false,
        operations: &ERRORS,
    },
    Resource {
        name: "incidents",
        singular: "incident",
        project_scoped: false,
        operations: &INCIDENTS,
    },
    Resource {
        name: "checks",
        singular: "check",
        project_scoped: true,
        operations: &CHECKS,
    },
    Resource {
        name: "heartbeats",
        singular: "heartbeat",
        project_scoped: true,
        operations: &HEARTBEATS,
    },
    Resource {
        name: "cloud-ip-ranges",
        singular: "cloud IP range",
        project_scoped: false,
        operations: &CLOUD_IP_RANGES,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_is_documented_in_its_help() {
        for resource in RESOURCES {
            let help = crate::help::text(resource.name);
            for operation in resource.operations {
                let usage = format!("goodissues {} {}", resource.name, operation.name);
                assert!(help.contains(&usage), "{usage} missing from help");
            }
        }
    }

    #[test]
    fn required_and_require_any_flags_are_accepted_flags() {
        for resource in RESOURCES {
            for operation in resource.operations {
                let mut flags: Vec<&str> = Vec::new();
                if let Some(query) = &operation.query {
                    flags.extend(query.require_any);
                }
                if let Body::Typed(typed) = &operation.body {
                    flags.extend(typed.required.iter().map(|(flag, _)| *flag));
                    flags.extend(typed.require_any);
                }
                for flag in flags {
                    assert!(
                        operation.accepts(resource, flag),
                        "{} {} {flag}",
                        resource.name,
                        operation.name
                    );
                }
            }
        }
    }
}
