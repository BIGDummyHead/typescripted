# RS-TS

RS-TS or "Rust to Typescript" is a proc macro library that is able to convert structs and enums to Typescript files.

It does this by generating a `types` folder and exports each exported type as `name.ts` into the folder.

## Usage

```rust
 #[derive(ExportTypescript)]
pub enum Roles {
    User,
    Admin,
    SuperAdmin,
}

#[derive(ExportTypescript)]
pub struct SuperUser {
    pub name: String,
    pub age: i32,
    pub roles: Roles,
    pub meta: Vec<String>,
}
```
 
### Output 

`./types/Roles.ts`

```ts

enum Roles {
	User = "User",
	Admin = "Admin",
	SuperAdmin = "SuperAdmin"
}

export default Roles
```

`./types/SuperUser.ts`

```ts
import Roles from './Roles';

interface SuperUser {
	name: string;
	age: number;
	roles: Roles;
	meta: Array<string>;
}

export default SuperUser
```

