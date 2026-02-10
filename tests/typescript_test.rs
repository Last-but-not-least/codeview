use codeview::{process_path, ProcessOptions, OutputFormat};
use std::io::Write;
use tempfile::NamedTempFile;

fn opts() -> ProcessOptions {
    ProcessOptions {
        symbols: vec![],
        pub_only: false,
        fns_only: false,
        types_only: false,
        no_tests: false,
        depth: None,
        format: OutputFormat::Plain,
        stats: false,
    }
}

fn write_ts(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".ts").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

fn write_tsx(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".tsx").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
    f
}

const SAMPLE_TS: &str = r#"import { EventEmitter } from "events";

export interface User {
    name: string;
    age: number;
    email?: string;
}

export type UserId = string | number;

export enum Role {
    Admin = "ADMIN",
    User = "USER",
    Guest = "GUEST",
}

export const MAX_USERS = 100;

export class UserService {
    private db: Map<string, User>;

    constructor() {
        this.db = new Map();
    }

    public getUser(id: string): User | undefined {
        return this.db.get(id);
    }

    public createUser(name: string, age: number): User {
        const user: User = { name, age };
        this.db.set(name, user);
        return user;
    }

    private validate(user: User): boolean {
        return user.name.length > 0 && user.age > 0;
    }
}

function helperFunction(x: number): number {
    return x * 2;
}

export function publicApi(input: string): string {
    return input.trim().toLowerCase();
}
"#;

// --- Interface mode ---

#[test]
fn ts_interface_mode_basic() {
    let f = write_ts(SAMPLE_TS);
    let output = process_path(f.path().to_str().unwrap(), opts()).unwrap();

    // All top-level items should appear
    assert!(output.contains("interface User"), "Missing interface User");
    assert!(output.contains("type UserId"), "Missing type alias UserId");
    assert!(output.contains("enum Role"), "Missing enum Role");
    assert!(output.contains("const MAX_USERS"), "Missing const");
    assert!(output.contains("class UserService"), "Missing class");
    assert!(output.contains("import"), "Missing import");
    assert!(output.contains("function helperFunction"), "Missing helperFunction");
    assert!(output.contains("function publicApi"), "Missing publicApi");
    // Bodies should be collapsed
    assert!(output.contains("{ ... }"), "Missing collapsed bodies");
}

// --- Expand mode ---

#[test]
fn ts_expand_symbol() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.symbols = vec!["publicApi".to_string()];
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    assert!(output.contains("function publicApi"), "Missing publicApi");
    assert!(output.contains("trim().toLowerCase()"), "Missing function body");
}

#[test]
fn ts_expand_class() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.symbols = vec!["UserService".to_string()];
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    assert!(output.contains("class UserService"), "Missing class");
    assert!(output.contains("new Map()") || output.contains("this.db"), "Missing class body");
}

// --- --pub filter ---

#[test]
fn ts_pub_filter() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.pub_only = true;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    // Exported items should appear
    assert!(output.contains("interface User"), "Missing exported interface");
    assert!(output.contains("function publicApi"), "Missing exported function");
    // Non-exported items should not
    assert!(!output.contains("helperFunction"), "Should not contain non-exported helperFunction");
}

// --- --fns filter ---

#[test]
fn ts_fns_filter() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.fns_only = true;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    // Functions and methods should appear
    assert!(output.contains("function helperFunction") || output.contains("function publicApi"),
            "Missing functions");
    // Types should not
    assert!(!output.contains("interface User"), "Should not contain interface");
    assert!(!output.contains("enum Role"), "Should not contain enum");
}

// --- --types filter ---

#[test]
fn ts_types_filter() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.types_only = true;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    assert!(output.contains("interface User"), "Missing interface");
    assert!(output.contains("enum Role"), "Missing enum");
    assert!(output.contains("type UserId"), "Missing type alias");
    assert!(output.contains("class UserService"), "Missing class");
    // Standalone functions should not appear
    assert!(!output.contains("function helperFunction"), "Should not contain standalone fn");
}

// --- --no-tests (no-op for TS, shouldn't break) ---

#[test]
fn ts_no_tests_noop() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.no_tests = true;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    assert!(output.contains("class UserService"), "Missing class with --no-tests");
    assert!(output.contains("interface User"), "Missing interface with --no-tests");
}

// --- Abstract class ---

#[test]
fn ts_abstract_class() {
    let src = r#"
export abstract class Shape {
    abstract area(): number;
    abstract perimeter(): number;

    public describe(): string {
        return `Area: ${this.area()}`;
    }
}
"#;
    let f = write_ts(src);
    let output = process_path(f.path().to_str().unwrap(), opts()).unwrap();

    assert!(output.contains("abstract class Shape"), "Missing abstract class");
    assert!(output.contains("area()"), "Missing abstract method area");
    assert!(output.contains("perimeter()"), "Missing abstract method perimeter");
    assert!(output.contains("describe()"), "Missing concrete method describe");
}

// --- TSX detection ---

#[test]
fn tsx_file_detection() {
    let src = r#"
import React from "react";

interface Props {
    name: string;
    count: number;
}

export function Greeting({ name, count }: Props): JSX.Element {
    return <div>Hello {name}, count: {count}</div>;
}

export class Counter extends React.Component<Props> {
    render() {
        return <span>{this.props.count}</span>;
    }
}
"#;
    let f = write_tsx(src);
    let output = process_path(f.path().to_str().unwrap(), opts()).unwrap();

    assert!(output.contains("interface Props"), "Missing interface in TSX");
    assert!(output.contains("function Greeting"), "Missing function in TSX");
    assert!(output.contains("class Counter"), "Missing class in TSX");
}

// --- --stats ---

#[test]
fn ts_stats_mode() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.stats = true;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    assert!(output.contains("files:"), "Missing files count in stats");
    assert!(output.contains("lines:"), "Missing lines count in stats");
    assert!(output.contains("items:"), "Missing items count in stats");
}

#[test]
fn ts_stats_json() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.stats = true;
    o.format = OutputFormat::Json;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&output)
        .expect("Stats JSON should be valid");
    assert!(parsed.is_object() || parsed.is_array(), "Should be structured JSON");
}

// --- Combined filters ---

#[test]
fn ts_pub_fns_combined() {
    let f = write_ts(SAMPLE_TS);
    let mut o = opts();
    o.pub_only = true;
    o.fns_only = true;
    let output = process_path(f.path().to_str().unwrap(), o).unwrap();

    assert!(output.contains("function publicApi"), "Missing exported function");
    assert!(!output.contains("helperFunction"), "Should not contain non-exported fn");
    assert!(!output.contains("interface User"), "Should not contain types");
}
