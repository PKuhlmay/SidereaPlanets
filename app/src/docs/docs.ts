import { Marked } from "marked";

export interface DocPage {
  slug: string;
  title: string;
  group: string;
  order: number;
  body: string;
}

export interface DocGroup {
  name: string;
  pages: DocPage[];
}

// A page starts with a small front matter block: title, group and order (position in the whole list).
const frontMatter = /^---\r?\n([\s\S]*?)\r?\n---\r?\n([\s\S]*)$/;

export function parsePage(slug: string, raw: string): DocPage {
  const match = frontMatter.exec(raw);
  if (!match) throw new Error(`Doc page "${slug}" has no front matter`);
  const fields = new Map<string, string>();
  for (const line of match[1].split(/\r?\n/)) {
    const colon = line.indexOf(":");
    if (colon > 0) fields.set(line.slice(0, colon).trim(), line.slice(colon + 1).trim());
  }
  const title = fields.get("title");
  if (!title) throw new Error(`Doc page "${slug}" has no title`);
  return {
    slug,
    title,
    group: fields.get("group") ?? "",
    order: Number(fields.get("order") ?? 0),
    body: match[2].replace(/\r\n/g, "\n"),
  };
}

export function groupPages(pages: DocPage[]): DocGroup[] {
  const groups: DocGroup[] = [];
  for (const page of [...pages].sort((a, b) => a.order - b.order)) {
    let group = groups.find((g) => g.name === page.group);
    if (!group) groups.push((group = { name: page.group, pages: [] }));
    group.pages.push(page);
  }
  return groups;
}

// Every word must appear in the title or the body; pages with a title hit come first.
export function searchPages(pages: DocPage[], query: string): DocPage[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  if (words.length === 0) return pages;
  const hits = pages.filter((p) => {
    const text = `${p.title}\n${p.body}`.toLowerCase();
    return words.every((w) => text.includes(w));
  });
  const inTitle = (p: DocPage) => words.some((w) => p.title.toLowerCase().includes(w));
  return [...hits.filter(inTitle), ...hits.filter((p) => !inTitle(p))];
}

// A link without a scheme or slash ("[Welcome](welcome)") points to another page of the documentation.
const marked = new Marked({
  renderer: {
    link({ href, text }) {
      if (/^[a-z0-9-]+$/.test(href)) return `<a href="#${href}" data-page="${href}">${text}</a>`;
      return false;
    },
    // Rows that name something as not available yet are greyed out (class "off").
    tablerow({ text }) {
      return `<tr${text.includes("Not available yet") ? ' class="off"' : ""}>\n${text}</tr>\n`;
    },
  },
});

export function renderBody(markdown: string): string {
  return marked.parse(markdown, { async: false });
}
