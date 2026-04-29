const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;
import renderMathInElement from "./assets/katex/contrib/auto-render.mjs";
import { marked } from "./assets/marked.esm.js"
const { getCurrentWindow } = window.__TAURI__.window

// 用于中断当前翻译请求（防抖/取消）
let currentAbortController = null;
let isAltPressed = false;
// 存储完整的Markdown内容
let fullMarkdownContent = "";
let dragbar = document.getElementById("dragbar");
let isAlwaysOnTop = false;
const currentWindow = getCurrentWindow();

async function translateText(text) {
  // 并行获取配置、prompt 指令和用户原始输入
  const [config, { prompt, input }] = await Promise.all([
    invoke('get_config'),
    invoke('get_translate_prompt', { text }),
  ]);

  const llm = config.select.llm;
  const providerConfig = config[llm];

  console.log(`text type detected, using ${llm}`);

  const apiBase = providerConfig.api_base.trim();
  const apiKey = providerConfig.api_key;
  const model = providerConfig.default_model;
  const temperature = providerConfig.temperature ?? 1.3;
  const thinking = providerConfig.thinking;

  const payload = {
    model: model,
    messages: [
      { role: "system", content: prompt },
      { role: "user", content: input }
    ],
    temperature: temperature,
    stream: true,
    thinking:{ "type": thinking }
  };

  // 取消上一个请求
  if (currentAbortController) {
    currentAbortController.abort();
  }
  currentAbortController = new AbortController();

  const Dom = document.getElementById("translate-content");
  Dom.innerHTML = "";
  fullMarkdownContent = "";

  try {
    const response = await fetch(`${apiBase}/chat/completions`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
      signal: currentAbortController.signal,
    });

    if (!response.ok || !response.body) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder('utf-8');
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // 处理可能的多行数据
      let lines = buffer.split('\n');
      buffer = lines.pop() || ""; // 保留不完整的最后一行

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          const dataStr = line.slice(6).trim();
          if (dataStr === '[DONE]') {
            reader.cancel();
            // 最终渲染一次
            await renderMarkdownWithMath(fullMarkdownContent);
            return;
          }

          try {
            const parsed = JSON.parse(dataStr);
            const content = parsed.choices?.[0]?.delta?.content || '';
            if (content) {
              await renderIncrementalMarkdown(content);
            }
          } catch (e) {
            console.warn('Failed to parse SSE data:', dataStr, e);
          }
        }
      }
    }
  } catch (error) {
    if (error.name === 'AbortError') {
      console.log('请求被取消');
    } else {
      console.error('翻译出错:', error);
      fullMarkdownContent = "## 翻译失败\n\n请检查网络连接或API配置。";
      await renderMarkdownWithMath(fullMarkdownContent);
    }
  }
}

// Markdown渲染函数
async function renderMarkdownWithMath(content) {
  const Dom = document.getElementById("translate-content");

  if (typeof marked !== 'undefined') {
    try {
      marked.setOptions({
        breaks: true,
        gfm: true,
      });

      const html = marked.parse(content);
      Dom.innerHTML = html;

      renderMathInElement(Dom, {
        delimiters: [
          { left: "$$", right: "$$", display: true },
          { left: "$", right: "$", display: false },
          { left: "\\(", right: "\\)", display: false },
          { left: "\\[", right: "\\]", display: true }
        ],
        throwOnError: false
      });
    } catch (error) {
      console.error('Markdown渲染错误:', error);
      Dom.textContent = content;
    }
  } else {
    Dom.textContent = content;
    renderMathInElement(Dom, {
      delimiters: [
        { left: "$$", right: "$$", display: true },
        { left: "$", right: "$", display: false },
        { left: "\\(", right: "\\)", display: false },
        { left: "\\[", right: "\\]", display: true }
      ],
      throwOnError: false
    });
  }
}

// 渐进式渲染函数
async function renderIncrementalMarkdown(newContent) {
  fullMarkdownContent += newContent;
  await renderMarkdownWithMath(fullMarkdownContent);
}


async function main() {
  const Dom = document.getElementById("translate-content");

  const unlisten = await listen('get_text', async (event) => {
    const originalText = event.payload;
    if (!originalText.trim()) {
      Dom.textContent = "";
      return;
    }

    // 执行翻译
    await translateText(originalText);
  });

  window.unlistenTranslate = unlisten;
}

window.addEventListener("DOMContentLoaded", () => {
  main();

  const closeBtn = document.getElementById("close-btn");
  const topBtn = document.getElementById("top-btn");

  closeBtn.addEventListener("click", async () => {
    invoke('hide_panel');
  });

  closeBtn.addEventListener("mousedown", (e) => {
    e.stopPropagation();
  });

  topBtn.addEventListener("click", async () => {
    isAlwaysOnTop = !isAlwaysOnTop;
    await currentWindow.setAlwaysOnTop(isAlwaysOnTop);
    topBtn.classList.toggle("active", isAlwaysOnTop);
  });

  topBtn.addEventListener("mousedown", (e) => {
    e.stopPropagation();
  });

});

window.addEventListener('keydown', (event) => {
  if (event.key === 'Escape') {
    invoke('hide_panel');
  }
});

window.addEventListener('keydown', (e) => {
  if (e.altKey && !isAltPressed) {
    isAltPressed = true;
  }
});

window.addEventListener('keyup', (e) => {
  if (e.key === 'Alt') {
    isAltPressed = false;
  }
});

// 拖拽栏事件
dragbar.addEventListener('mousedown', (e) => {
  invoke('start_drag');
  if (e.button === 0 && isAltPressed) {
    e.preventDefault();
    invoke('start_drag');
  }
});
