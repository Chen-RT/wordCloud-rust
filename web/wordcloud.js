// wordcloud.js
import init, { WordCloud } from './pkg/wordcloud.js';

export class WordCloudRenderer {
  constructor(canvasId, options = {}) {
    this.canvas = document.getElementById(canvasId);
    if (!this.canvas) {
      throw new Error(`Canvas with id "${canvasId}" not found`);
    }
    
    this.ctx = this.canvas.getContext('2d');
    this.width = this.canvas.width;
    this.height = this.canvas.height;
    
    // Default options
    this.options = {
      fontFamily: options.fontFamily || 'Arial, sans-serif',
      fontWeight: options.fontWeight || 'normal',
      minSize: options.minSize || 10,
      maxSize: options.maxSize || 60,
      rotationRange: options.rotationRange || 0, // 0 for no rotation
      spiral: options.spiral || 'archimedean', // 'archimedean' or 'rectangular'
      colors: options.colors || ['#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd']
    };
    
    this.wasmLoaded = false;
    this.wordcloudInstance = null;
    
    // Initialize WASM
    this.initWasm();
  }
  
  async initWasm() {
    try {
      await init();
      
      // Create WordCloud instance
      this.wordcloudInstance = new WordCloud(
        this.width,
        this.height,
        this.options.fontFamily,
        this.options.fontWeight,
        this.options.minSize,
        this.options.maxSize
      );
      
      // Set additional options
      this.wordcloudInstance.set_rotation_range(this.options.rotationRange);
      this.wordcloudInstance.set_spiral(this.options.spiral);
      
      this.wasmLoaded = true;
      console.log('WordCloud WASM module loaded successfully');
    } catch (error) {
      console.error('Failed to load WordCloud WASM module:', error);
    }
  }
  
  async generate(words) {
    if (!this.wasmLoaded || !this.wordcloudInstance) {
      console.error('WASM module not loaded');
      return;
    }
    
    // Prepare words input
    const wordsWithColor = words.map((word, index) => {
      if (typeof word === 'string') {
        return {
          text: word,
          weight: 1,
          color: this.getColor(index)
        };
      } else if (Array.isArray(word)) {
        return {
          text: word[0],
          weight: word[1] || 1,
          color: word[2] || this.getColor(index)
        };
      } else {
        return {
          text: word.text,
          weight: word.weight || 1,
          color: word.color || this.getColor(index),
          rotate: word.rotate
        };
      }
    });
    
    // Convert to JSON for WASM
    const wordsJson = JSON.stringify(wordsWithColor);
    
    // Generate layout
    const layoutJson = this.wordcloudInstance.generate_layout(wordsJson);
    const layout = JSON.parse(layoutJson);
    
    // Clear canvas
    this.ctx.clearRect(0, 0, this.width, this.height);
    
    // Draw words
    layout.forEach(word => {
      this.drawWord(word);
    });
    
    return layout;
  }
  
  drawWord(word) {
    const { text, x, y, rotate, color, size } = word;
    
    this.ctx.save();
    
    // Set font
    this.ctx.font = `${this.options.fontWeight} ${size}px ${this.options.fontFamily}`;
    
    // Set color
    this.ctx.fillStyle = color || '#000000';
    
    // Position and rotate
    this.ctx.translate(x, y);
    this.ctx.rotate(rotate * Math.PI / 180);
    
    // Draw text
    this.ctx.textAlign = 'center';
    this.ctx.textBaseline = 'middle';
    this.ctx.fillText(text, 0, 0);
    
    this.ctx.restore();
  }
  
  getColor(index) {
    return this.options.colors[index % this.options.colors.length];
  }
}