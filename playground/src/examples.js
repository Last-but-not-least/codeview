export const examples = {
  rust: [
    {
      name: 'User Service',
      code: `use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.email.contains('@')
    }

    fn validate_email(&self) -> bool {
        self.email.contains('@') && self.email.contains('.')
    }
}

pub struct UserService {
    users: HashMap<u64, User>,
}

impl UserService {
    pub fn new() -> Self {
        Self { users: HashMap::new() }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), String> {
        if self.users.contains_key(&user.id) {
            return Err("User already exists".to_string());
        }
        self.users.insert(user.id, user);
        Ok(())
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }
}`
    },
    {
      name: 'API Router',
      code: `use axum::{Router, routing::get, Json};

pub enum ApiError {
    NotFound,
    BadRequest(String),
}

pub struct ApiRouter {
    port: u16,
}

impl ApiRouter {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn serve(self) -> Result<(), ApiError> {
        let app = Router::new()
            .route("/", get(root))
            .route("/users", get(list_users));
        Ok(())
    }
}

async fn root() -> &'static str {
    "API is running"
}

async fn list_users() -> Json<Vec<String>> {
    Json(vec!["Alice".to_string()])
}`
    },
    {
      name: 'Data Model',
      code: `use uuid::Uuid;

pub trait Identifiable {
    fn id(&self) -> Uuid;
}

#[derive(Debug, Clone)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    published: bool,
}

impl Post {
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            published: false,
        }
    }

    pub fn publish(&mut self) {
        self.published = true;
    }
}

impl Identifiable for Post {
    fn id(&self) -> Uuid {
        self.id
    }
}`
    }
  ],
  typescript: [
    {
      name: 'User Service',
      code: `export interface User {
    id: number;
    name: string;
    email: string;
}

export type UserId = string | number;

export class UserService {
    private cache: Map<UserId, User>;

    constructor() {
        this.cache = new Map();
    }

    public async getUser(id: UserId): Promise<User | undefined> {
        const cached = this.cache.get(id);
        if (cached) {
            return cached;
        }
        return undefined;
    }

    public async createUser(name: string, email: string): Promise<User> {
        const user: User = {
            id: Math.floor(Math.random() * 100000),
            name,
            email,
        };
        this.cache.set(user.id, user);
        return user;
    }

    private clearCache(): void {
        this.cache.clear();
    }
}

export function parseUserId(raw: string): UserId {
    const num = parseInt(raw, 10);
    return isNaN(num) ? raw : num;
}`
    },
    {
      name: 'API Router',
      code: `import express from "express";

export interface RouteConfig {
    path: string;
    method: "GET" | "POST";
}

export class Router {
    private app: express.Application;

    constructor(prefix: string = "/api") {
        this.app = express();
    }

    public get(path: string, handler: Function): this {
        return this;
    }

    public post(path: string, handler: Function): this {
        return this;
    }

    public listen(port: number): void {
        this.app.listen(port, () => {
            console.log(\`Server on port \${port}\`);
        });
    }
}`
    },
    {
      name: 'Data Model',
      code: `export interface Identifiable {
    id: string;
}

export type Status = "draft" | "published";

export interface Post extends Identifiable {
    title: string;
    content: string;
    status: Status;
}

export class PostManager {
    private posts: Map<string, Post>;

    constructor() {
        this.posts = new Map();
    }

    public create(title: string, content: string): Post {
        const post: Post = {
            id: this.generateId(),
            title,
            content,
            status: "draft",
        };
        this.posts.set(post.id, post);
        return post;
    }

    public publish(id: string): boolean {
        const post = this.posts.get(id);
        if (!post) return false;
        post.status = "published";
        return true;
    }

    private generateId(): string {
        return Math.random().toString(36).substring(2, 15);
    }
}`
    }
  ],
  python: [
    {
      name: 'User Service',
      code: `from dataclasses import dataclass
from typing import Optional, Dict

@dataclass
class User:
    id: int
    name: str
    email: str
    _password: str

    def is_valid(self) -> bool:
        return bool(self.name) and '@' in self.email

    def _hash_password(self, password: str) -> str:
        import hashlib
        return hashlib.sha256(password.encode()).hexdigest()

class UserService:
    def __init__(self):
        self._users: Dict[int, User] = {}

    def get_user(self, user_id: int) -> Optional[User]:
        return self._users.get(user_id)

    def create_user(self, name: str, email: str) -> User:
        user = User(len(self._users) + 1, name, email, "")
        self._users[user.id] = user
        return user

    def _validate(self, user: User) -> bool:
        return user.is_valid()

def create_service() -> UserService:
    return UserService()`
    },
    {
      name: 'API Router',
      code: `from flask import Flask, jsonify
from typing import Callable

class Router:
    def __init__(self, prefix: str = "/api"):
        self.app = Flask(__name__)
        self.prefix = prefix

    def get(self, path: str):
        def decorator(handler: Callable):
            return handler
        return decorator

    def post(self, path: str):
        def decorator(handler: Callable):
            return handler
        return decorator

    def run(self, port: int = 5000):
        self.app.run(port=port)

def create_app() -> Router:
    router = Router()

    @router.get("/health")
    def health():
        return jsonify({"status": "ok"})

    return router`
    },
    {
      name: 'Data Model',
      code: `from dataclasses import dataclass
from uuid import UUID, uuid4

@dataclass
class Post:
    id: UUID
    title: str
    content: str
    published: bool = False

    @classmethod
    def create(cls, title: str, content: str) -> "Post":
        return cls(
            id=uuid4(),
            title=title,
            content=content,
        )

    def publish(self) -> None:
        self.published = True

    def _validate(self) -> bool:
        return bool(self.title) and bool(self.content)

class PostManager:
    def __init__(self):
        self._posts: dict[UUID, Post] = {}

    def add(self, post: Post) -> None:
        if post._validate():
            self._posts[post.id] = post

    def get(self, post_id: UUID) -> Post | None:
        return self._posts.get(post_id)`
    }
  ],
  javascript: [
    {
      name: 'User Service',
      code: `export class UserService {
    constructor() {
        this.cache = new Map();
    }

    async getUser(id) {
        const cached = this.cache.get(id);
        if (cached) {
            return cached;
        }
        return null;
    }

    async createUser(name, email) {
        const user = {
            id: Math.floor(Math.random() * 100000),
            name,
            email,
        };
        this.cache.set(user.id, user);
        return user;
    }

    clearCache() {
        this.cache.clear();
    }
}

export function parseUserId(raw) {
    const num = parseInt(raw, 10);
    return isNaN(num) ? raw : num;
}

function validateEmail(email) {
    return email.includes("@");
}`
    },
    {
      name: 'API Router',
      code: `import express from "express";

export class Router {
    constructor(prefix = "/api") {
        this.app = express();
        this.prefix = prefix;
    }

    get(path, handler) {
        return this;
    }

    post(path, handler) {
        return this;
    }

    listen(port) {
        this.app.listen(port, () => {
            console.log(\`Server on \${port}\`);
        });
    }
}

export function createApp(config) {
    const router = new Router();
    return router;
}

function validateConfig(config) {
    return config && config.port;
}`
    },
    {
      name: 'Data Model',
      code: `export class Post {
    constructor(title, content) {
        this.id = this.generateId();
        this.title = title;
        this.content = content;
        this.published = false;
    }

    publish() {
        this.published = true;
    }

    validate() {
        return this.title && this.content;
    }

    generateId() {
        return Math.random().toString(36).substring(2, 15);
    }
}

export class PostManager {
    constructor() {
        this.posts = new Map();
    }

    add(post) {
        if (post.validate()) {
            this.posts.set(post.id, post);
            return true;
        }
        return false;
    }

    get(postId) {
        return this.posts.get(postId);
    }
}

function loadDefaults() {
    return { maxPosts: 1000 };
}`
    }
  ]
}
