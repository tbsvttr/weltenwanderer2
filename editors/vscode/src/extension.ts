import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("weltenwanderer");
  const lspPath = config.get<string>("lspPath", "ww-lsp");

  const serverOptions: ServerOptions = {
    command: lspPath,
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "ww" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.ww"),
    },
  };

  client = new LanguageClient(
    "weltenwanderer",
    "Weltenwanderer Language Server",
    serverOptions,
    clientOptions
  );

  client.start();

  // --- Custom commands ---

  context.subscriptions.push(
    vscode.commands.registerCommand("weltenwanderer.restartServer", async () => {
      if (client) {
        await client.stop();
        await client.start();
        vscode.window.showInformationMessage("Weltenwanderer language server restarted.");
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("weltenwanderer.showOutput", () => {
      if (client) {
        client.outputChannel.show();
      }
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("weltenwanderer.newWorld", async () => {
      const name = await vscode.window.showInputBox({
        prompt: "World name",
        placeHolder: "The Iron Kingdoms",
      });
      if (!name) return;

      const folders = vscode.workspace.workspaceFolders;
      const baseUri = folders?.[0]?.uri;
      if (!baseUri) {
        vscode.window.showErrorMessage("Open a workspace folder first.");
        return;
      }

      const fileName = name.toLowerCase().replace(/\s+/g, "-") + ".ww";
      const fileUri = vscode.Uri.joinPath(baseUri, fileName);

      const content = `world "${name}" {\n    genre "fantasy"\n    setting ""\n}\n`;
      await vscode.workspace.fs.writeFile(fileUri, Buffer.from(content, "utf-8"));
      const doc = await vscode.workspace.openTextDocument(fileUri);
      await vscode.window.showTextDocument(doc);
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("weltenwanderer.newEntity", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "ww") {
        vscode.window.showErrorMessage("Open a .ww file first.");
        return;
      }

      const kind = await vscode.window.showQuickPick(
        [
          "character",
          "location",
          "fortress",
          "region",
          "city",
          "dungeon",
          "faction",
          "item",
          "event",
          "lore",
        ],
        { placeHolder: "Select entity kind" }
      );
      if (!kind) return;

      const name = await vscode.window.showInputBox({
        prompt: `Name for the new ${kind}`,
        placeHolder: entityNamePlaceholder(kind),
      });
      if (!name) return;

      const snippet = buildEntitySnippet(kind, name);
      const position = editor.selection.active;
      await editor.insertSnippet(snippet, position);
    })
  );
}

function entityNamePlaceholder(kind: string): string {
  switch (kind) {
    case "character": return "Kael Stormborn";
    case "location":
    case "fortress":
    case "region":
    case "city":
    case "dungeon": return "the Iron Citadel";
    case "faction": return "the Order of Dawn";
    case "item": return "the Blade of First Light";
    case "event": return "the Great Sundering";
    case "lore": return "the Prophecy of Second Dawn";
    default: return "Name";
  }
}

function buildEntitySnippet(kind: string, name: string): vscode.SnippetString {
  const article = needsArticle(kind) ? "a " : "an ";

  switch (kind) {
    case "character":
      return new vscode.SnippetString(
        `\n${name} is a character {\n    species \${1:human}\n    occupation \${2:adventurer}\n    status \${3|alive,dead,missing|}\n    traits [\${4:brave, loyal}]\n\n    \"\"\"\n    \${5:Description.}\n    \"\"\"\n}\n`
      );
    case "fortress":
    case "region":
    case "city":
    case "dungeon":
      return new vscode.SnippetString(
        `\n${name} is ${article}${kind} {\n    climate \${1:temperate}\n\n    \"\"\"\n    \${2:Description.}\n    \"\"\"\n}\n`
      );
    case "location":
      return new vscode.SnippetString(
        `\n${name} is a \${1|fortress,region,city,dungeon|} {\n    climate \${2:temperate}\n\n    \"\"\"\n    \${3:Description.}\n    \"\"\"\n}\n`
      );
    case "faction":
      return new vscode.SnippetString(
        `\n${name} is a faction {\n    type \${1:organization}\n    led by \${2:Leader}\n    values [\${3:honor, duty}]\n\n    \"\"\"\n    \${4:Description.}\n    \"\"\"\n}\n`
      );
    case "item":
      return new vscode.SnippetString(
        `\n${name} is an item {\n    type \${1|weapon,armor,artifact,tool,potion|}\n    rarity \${2|common,uncommon,rare,legendary|}\n\n    \"\"\"\n    \${3:Description.}\n    \"\"\"\n}\n`
      );
    case "event":
      return new vscode.SnippetString(
        `\n${name} is an event {\n    date year \${1:1}, era "\${2:Common Era}"\n    type \${3:historical}\n    involving [\${4:Participant}]\n\n    \"\"\"\n    \${5:Description.}\n    \"\"\"\n}\n`
      );
    case "lore":
      return new vscode.SnippetString(
        `\n${name} is a lore {\n    type \${1|prophecy,history,legend,myth|}\n    source "\${2:Unknown}"\n    references [\${3:Related Entity}]\n\n    \"\"\"\n    \${4:Content.}\n    \"\"\"\n}\n`
      );
    default:
      return new vscode.SnippetString(
        `\n${name} is ${article}${kind} {\n    \${1:property} \${2:value}\n\n    \"\"\"\n    \${3:Description.}\n    \"\"\"\n}\n`
      );
  }
}

function needsArticle(kind: string): boolean {
  // "an" before vowel sounds, "a" otherwise
  return !/^[aeiou]/i.test(kind);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
