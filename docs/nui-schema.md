# `nui-schema.yaml` Contract Guide

The `nui-schema.yaml` file is the Single Source of Truth (SSOT) contract for FiveM NUI (Network User Interface) development using `trek`. It defines data models, enums, one-way events, and two-way RPC endpoints shared between your React frontend and FiveM Lua client.

Running `trek codegen` reads this contract and automatically generates:
1. **TypeScript React Hooks & Types** using `@trekscripts/core/query`.
2. **Lua Event Emitters, Callback Handlers & Type Annotations** (EmmyLua / LuaLS compatible).

---

## Table of Contents

1. [Schema Overview](#schema-overview)
2. [Root Properties](#root-properties)
3. [Type System](#type-system)
4. [Defining Enums (`enums`)](#defining-enums-enums)
5. [Defining Events (`events`)](#defining-events-events)
6. [Defining Endpoints (`endpoints`)](#defining-endpoints-endpoints)
7. [Full Real-World Example](#full-real-world-example)
8. [Generated Code Output](#generated-code-output)
9. [IDE Schema Validation & Autocompletion](#ide-schema-validation--autocompletion)
10. [CLI Usage](#cli-usage)

---

## Schema Overview

A basic `nui-schema.yaml` file looks like this:

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/merzen-sh/trek-tools/refs/heads/main/trek-nui.schema.json

version: "1.0"
resource: "my_resource"

enums: []
events: []
endpoints: []
```

---

## Root Properties

| Property | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `version` | `string` | **Yes** | Contract version (e.g. `"1.0"`). |
| `resource` | `string` | **Yes** | The FiveM resource name. |
| `enums` | `array` | No | List of enum definitions. |
| `events` | `array` | No | List of one-way event definitions (Lua ? UI). |
| `endpoints` | `array` | No | List of two-way RPC endpoints (UI ? Lua). |

---

## Type System

`trek-codegen` supports a flexible, polymorphic type system for payloads, requests, and responses.

### 1. Primitive Types
- `string`: Text values.
- `number`: Numeric values (integers or floating-point numbers).
- `boolean`: `true` or `false`.

### 2. Optional Types (`?`)
Append `?` to make any primitive or enum type optional:
- `string?` ? TypeScript `string | undefined`, Lua doc `string`
- `number?` ? TypeScript `number | undefined`, Lua doc `number`
- `PlayerState?` ? TypeScript `PlayerState | undefined`, Lua doc `PlayerState`

### 3. Array Types (`[...]`)
Wrap a type inside square brackets to indicate a list / array:
- `[string]` ? TypeScript `Array<string>`, Lua doc `string[]`
- `[number]` ? TypeScript `Array<number>`, Lua doc `number[]`
- `[ItemRarity]` ? TypeScript `Array<ItemRarity>`, Lua doc `ItemRarity[]`
- `[{ id: string, count: number }]` ? Array of inline objects

### 4. Object Types
Define key-value maps inline or nested:
```yaml
payload:
  id: string
  count: number
  metadata:
    label: string
    durability: number?
```

### 5. Custom Enums
Reference any named enum defined under `enums:` directly:
```yaml
payload:
  rarity: ItemRarity
  previous_rarity: ItemRarity?
  all_rarities: [ItemRarity]
```

---

## Defining Enums (`enums`)

Enums allow you to define a set of named constants that are shared across TypeScript and Lua.

### Enum Schema

| Property | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `name` | `string` | **Yes** | Name of the enum (PascalCase recommended). |
| `description` | `string` | No | Documentation comment for the enum. |
| `values` | `array of string` | **Yes** | List of string values for the enum. |

### Example

```yaml
enums:
  - name: ItemRarity
    description: "Item rarity classification"
    values:
      - common
      - rare
      - epic
      - legendary

  - name: PlayerState
    description: "Current player status in game"
    values:
      - idle
      - in_combat
      - fast-travel
      - dead
```

### Generated Code for Enums

**TypeScript:**
```typescript
/**
 * Item rarity classification
 */
export enum ItemRarity {
  Common = "common",
  Rare = "rare",
  Epic = "epic",
  Legendary = "legendary",
}
```

**Lua:**
```lua
--- Item rarity classification
---@enum ItemRarity
NUI.ItemRarity = {
    Common = "common",
    Rare = "rare",
    Epic = "epic",
    Legendary = "legendary",
}
```

> **Note:** Member names are automatically converted to PascalCase (e.g. `in_combat` ? `InCombat`, `fast-travel` ? `FastTravel`), while preserving the exact string values.

---

## Defining Events (`events`)

Events represent **one-way communication** sent from the FiveM Lua client script to the React NUI frontend (`SendNUIMessage`).

### Event Schema

| Property | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `name` | `string` | **Yes** | Event action name (camelCase recommended). |
| `description` | `string` | No | Documentation for the event. |
| `payload` | `TypeDefinition` | **Yes** | Data structure sent with the event. |

### Example

```yaml
events:
  - name: playerStateUpdated
    description: "Emitted when the player status changes"
    payload:
      state: PlayerState
      previous_state: PlayerState?
      health: number
```

### Generated Code

- **TypeScript:** `usePlayerStateUpdatedEvent(handler: (data: PlayerStateUpdatedPayload) => void)`
- **Lua:** `NUI.emitPlayerStateUpdated(payload)`

---

## Defining Endpoints (`endpoints`)

Endpoints represent **two-way RPC communication** between the React frontend and FiveM Lua client (`RegisterNuiCallback` / `fetchNui`).

### Endpoint Schema

| Property | Type | Required | Description |
| :--- | :--- | :--- | :--- |
| `name` | `string` | **Yes** | Endpoint name. |
| `type` | `query` \| `mutation` | **Yes** | Endpoint classification: `query` for fetching data, `mutation` for actions / state changes. |
| `description` | `string` | No | Documentation for the endpoint. |
| `request` | `TypeDefinition` | No | Optional payload sent from UI to Lua. |
| `response` | `TypeDefinition` | **Yes** | Data structure returned from Lua back to UI. |

### Example

```yaml
endpoints:
  - name: getInventory
    type: query
    description: "Fetch the player inventory items"
    response:
      items:
        - name: string
          count: number
          rarity: ItemRarity

  - name: useItem
    type: mutation
    description: "Request to use an inventory item"
    request:
      item_name: string
      amount: number?
    response:
      success: boolean
      message: string?
```

### Generated Code

- **TypeScript:**
  - `useGetInventoryQuery(options?: NUIQueryOptions<void>)`
  - `useUseItemMutation(options?: NUIQueryOptions<UseItemRequest>)`
- **Lua:**
  - `NUI.onGetInventory(handler: fun(data: nil): GetInventoryResponse)`
  - `NUI.onUseItem(handler: fun(data: UseItemRequest): UseItemResponse)`

---

## Full Real-World Example

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/merzen-sh/trek-tools/refs/heads/main/trek-nui.schema.json

version: "1.0"
resource: "trek_inventory"

enums:
  - name: ItemType
    description: "Categories of inventory items"
    values:
      - weapon
      - ammo
      - consumable
      - misc

  - name: TransactionStatus
    description: "Status of an item transaction"
    values:
      - pending
      - completed
      - failed

events:
  - name: itemReceived
    description: "Notify UI when an item is added to the player"
    payload:
      item_id: string
      type: ItemType
      amount: number
      metadata:
        durability: number?

endpoints:
  - name: fetchInventory
    type: query
    description: "Get all items in the player inventory"
    response:
      items:
        - id: string
          name: string
          item_type: ItemType
          quantity: number

  - name: transferItem
    type: mutation
    description: "Transfer an item to another player"
    request:
      target_player_id: number
      item_id: string
      amount: number
    response:
      status: TransactionStatus
      success: boolean
```

---

## Generated Code Output

### Generated TypeScript (`react/src/generated/nui.ts`)

```typescript
// @ts-nocheck
// Generated by trek codegen. DO NOT EDIT DIRECTLY.

import { useNUIQuery, useNUIMutation, useNUIEvent, NUIQueryOptions } from "@trekscripts/core/query";

/**
 * Categories of inventory items
 */
export enum ItemType {
  Weapon = "weapon",
  Ammo = "ammo",
  Consumable = "consumable",
  Misc = "misc",
}

/**
 * Status of an item transaction
 */
export enum TransactionStatus {
  Pending = "pending",
  Completed = "completed",
  Failed = "failed",
}

/**
 * Notify UI when an item is added to the player
 */
export interface ItemReceivedPayload {
  amount: number;
  item_id: string;
  metadata: {
    durability?: number;
  };
  type: ItemType;
}

/**
 * Get all items in the player inventory
 */
export interface FetchInventoryResponse {
  items: Array<{
    id: string;
    item_type: ItemType;
    name: string;
    quantity: number;
  }>;
}

/**
 * Transfer an item to another player
 */
export interface TransferItemRequest {
  amount: number;
  item_id: string;
  target_player_id: number;
}

/**
 * Transfer an item to another player
 */
export interface TransferItemResponse {
  status: TransactionStatus;
  success: boolean;
}

/**
 * Notify UI when an item is added to the player
 */
export function useItemReceivedEvent(handler: (data: ItemReceivedPayload) => void) {
  return useNUIEvent<ItemReceivedPayload>("itemReceived", handler);
}

/**
 * Get all items in the player inventory
 */
export function useFetchInventoryQuery(options?: NUIQueryOptions<void>) {
  return useNUIQuery<FetchInventoryResponse, void>("fetchInventory", options);
}

/**
 * Transfer an item to another player
 */
export function useTransferItemMutation(options?: NUIQueryOptions<TransferItemRequest>) {
  return useNUIMutation<TransferItemResponse, TransferItemRequest>("transferItem", options);
}
```

### Generated Lua (`src/shared/nui_events.lua`)

```lua
-- Generated by trek codegen. DO NOT EDIT DIRECTLY.
NUI = {}

--- Categories of inventory items
---@enum ItemType
NUI.ItemType = {
    Weapon = "weapon",
    Ammo = "ammo",
    Consumable = "consumable",
    Misc = "misc",
}

--- Status of an item transaction
---@enum TransactionStatus
NUI.TransactionStatus = {
    Pending = "pending",
    Completed = "completed",
    Failed = "failed",
}

--- Notify UI when an item is added to the player
---@param payload { amount: number, item_id: string, metadata: { durability: number }, type: ItemType }
function NUI.emitItemReceived(payload)
    SendNUIMessage({
        action = "itemReceived",
        data = payload
    })
end

--- Get all items in the player inventory
---@param handler fun(data: nil): { items: { id: string, item_type: ItemType, name: string, quantity: number }[] }
function NUI.onFetchInventory(handler)
    RegisterNuiCallback("fetchInventory", function(data, cb)
        local response = handler(data)
        cb(response or {})
    end)
end

--- Transfer an item to another player
---@param handler fun(data: { amount: number, item_id: string, target_player_id: number }): { status: TransactionStatus, success: boolean }
function NUI.onTransferItem(handler)
    RegisterNuiCallback("transferItem", function(data, cb)
        local response = handler(data)
        cb(response or {})
    end)
end
```

---

## IDE Schema Validation & Autocompletion

To get real-time validation and autocompletion in VSCode, Cursor, or Neovim (via `yaml-language-server`), add the schema comment at the top of your `nui-schema.yaml`:

```yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/merzen-sh/trek-tools/refs/heads/main/trek-nui.schema.json
```

Or reference a local copy:
```yaml
# yaml-language-server: $schema=./trek-nui.schema.json
```

---

## CLI Usage

### 1. Initialize a Starter Schema
Generate a starter `nui-schema.yaml` and `trek-nui.schema.json`:
```bash
trek codegen --init-schema
```

### 2. Run Code Generation
```bash
# Generate using default paths (./react/src/generated/nui.ts and ./src/shared/nui_events.lua)
trek codegen

# Custom input/output paths
trek codegen -s ./schemas/nui.yaml -t ./web/src/nui.ts -l ./shared/nui.lua
```