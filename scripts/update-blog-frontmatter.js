#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const blogsDir = path.join(__dirname, '../docs/zh/Blogs');
const enBlogsDir = path.join(__dirname, '../docs/en/Blogs');

function updateFrontmatter(filePath) {
  let content = fs.readFileSync(filePath, 'utf-8');
  
  // 检查是否有旧的 frontmatter
  if (content.startsWith('---')) {
    // 替换 frontmatter
    content = content.replace(
      /^---\nlayout: blog-post\n---\n\n(# .+)/m,
      `---
layout: doc
sidebar: false
aside: false
pageClass: blog-post-page
---

$1

<BlogPost />`
    );
  } else {
    // 添加新的 frontmatter
    const titleMatch = content.match(/^(# .+)$/m);
    if (titleMatch) {
      content = content.replace(
        titleMatch[0],
        `---
layout: doc
sidebar: false
aside: false
pageClass: blog-post-page
---

${titleMatch[0]}

<BlogPost />`
      );
    }
  }
  
  fs.writeFileSync(filePath, content, 'utf-8');
  console.log(`Updated: ${path.basename(filePath)}`);
}

function processDirectory(dir) {
  if (!fs.existsSync(dir)) {
    console.log(`Directory not found: ${dir}`);
    return;
  }
  
  const files = fs.readdirSync(dir);
  
  files.forEach(file => {
    if (file.endsWith('.md')) {
      const filePath = path.join(dir, file);
      updateFrontmatter(filePath);
    }
  });
}

console.log('Updating Chinese blogs...');
processDirectory(blogsDir);

console.log('\nUpdating English blogs...');
processDirectory(enBlogsDir);

console.log('\nDone!');
