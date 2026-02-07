#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const blogsDir = path.join(__dirname, '../docs/zh/Blogs');
const enBlogsDir = path.join(__dirname, '../docs/en/Blogs');

function removeBlogPostTag(filePath) {
  let content = fs.readFileSync(filePath, 'utf-8');
  
  // 移除 <BlogPost /> 标签
  content = content.replace(/\n\n<BlogPost \/>\n/g, '\n');
  content = content.replace(/<BlogPost \/>/g, '');
  
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
      removeBlogPostTag(filePath);
    }
  });
}

console.log('Removing <BlogPost /> tags from Chinese blogs...');
processDirectory(blogsDir);

console.log('\nRemoving <BlogPost /> tags from English blogs...');
processDirectory(enBlogsDir);

console.log('\nDone!');
