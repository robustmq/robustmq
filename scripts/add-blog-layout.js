#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const blogsDir = path.join(__dirname, '../docs/zh/Blogs');
const enBlogsDir = path.join(__dirname, '../docs/en/Blogs');

function addFrontmatter(filePath) {
  const content = fs.readFileSync(filePath, 'utf-8');
  
  // 检查是否已经有 frontmatter
  if (content.startsWith('---')) {
    console.log(`Skipped: ${path.basename(filePath)} (already has frontmatter)`);
    return;
  }
  
  // 添加 frontmatter
  const newContent = `---
layout: blog-post
---

${content}`;
  
  fs.writeFileSync(filePath, newContent, 'utf-8');
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
      addFrontmatter(filePath);
    }
  });
}

console.log('Adding frontmatter to Chinese blogs...');
processDirectory(blogsDir);

console.log('\nAdding frontmatter to English blogs...');
processDirectory(enBlogsDir);

console.log('\nDone!');
