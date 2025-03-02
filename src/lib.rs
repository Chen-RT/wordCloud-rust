use js_sys::{Array, Math, Object, Reflect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, TextMetrics, console};

// 词条数据结构
#[derive(Serialize, Deserialize)]
pub struct WordItem {
    text: String,
    weight: f64,
    // 可选字段
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rotate: Option<f64>,
}

// 位置数据结构
#[derive(Serialize, Deserialize)]
pub struct WordPosition {
    text: String,
    weight: f64,
    x: f64,
    y: f64,
    rotate: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<f64>,
}

// 配置选项
#[derive(Serialize, Deserialize)]
pub struct CloudOptions {
    width: u32,
    height: u32,
    font_family: String,
    font_weight: String,
    min_size: f64,
    max_size: f64,
    #[serde(default = "default_rotation_range")]
    rotation_range: f64,
    #[serde(default = "default_spiral")]
    spiral: String,
}

fn default_rotation_range() -> f64 {
    0.0
}

fn default_spiral() -> String {
    "archimedean".to_string()
}

#[wasm_bindgen]
pub struct WordCloud {
    options: CloudOptions,
    grid: Vec<Vec<bool>>,
    grid_size: usize,
}

#[wasm_bindgen]
impl WordCloud {
    #[wasm_bindgen(constructor)]
    pub fn new(
        width: u32,
        height: u32,
        font_family: String,
        font_weight: String,
        min_size: f64,
        max_size: f64,
    ) -> WordCloud {
        // 启用调试功能
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let options = CloudOptions {
            width,
            height,
            font_family,
            font_weight,
            min_size,
            max_size,
            rotation_range: 0.0,
            spiral: "archimedean".to_string(),
        };

        // 网格大小 - 调整为更精细以提高精度
        let grid_size = 4;
        let grid_width = (width as usize / grid_size) + 1;
        let grid_height = (height as usize / grid_size) + 1;

        let grid = vec![vec![false; grid_height]; grid_width];

        // 记录初始化信息
        console::log_1(&JsValue::from_str(&format!(
            "WordCloud initialized: {}x{} with grid {}x{}",
            width, height, grid_width, grid_height
        )));

        WordCloud {
            options,
            grid,
            grid_size,
        }
    }

    // 添加一个重置网格的方法
    #[wasm_bindgen]
    pub fn reset_grid(&mut self) -> bool {
        let width = self.options.width;
        let height = self.options.height;

        // 重新创建网格而不是清空现有网格
        let grid_width = (width as usize / self.grid_size) + 1;
        let grid_height = (height as usize / self.grid_size) + 1;

        self.grid = vec![vec![false; grid_height]; grid_width];

        // 记录重置信息
        console::log_1(&JsValue::from_str(&format!(
            "Grid reset to {}x{}",
            grid_width, grid_height
        )));

        // 返回成功标志
        true
    }

    // 设置旋转范围
    #[wasm_bindgen]
    pub fn set_rotation_range(&mut self, rotation_range: f64) {
        self.options.rotation_range = rotation_range;
    }

    // 设置螺旋类型
    #[wasm_bindgen]
    pub fn set_spiral(&mut self, spiral: String) {
        self.options.spiral = spiral;
    }

    // 生成词云布局
    #[wasm_bindgen]
    pub fn generate_layout(&mut self, words_json: String) -> String {
        // 记录生成开始
        console::log_1(&JsValue::from_str("开始生成词云布局"));

        // 重置网格
        let reset_success = self.reset_grid();
        console::log_1(&JsValue::from_str(&format!(
            "网格重置状态: {}",
            if reset_success { "成功" } else { "失败" }
        )));

        // 解析输入词语
        let words: Vec<WordItem> = match serde_json::from_str(&words_json) {
            Ok(w) => w,
            Err(e) => {
                console::log_1(&JsValue::from_str(&format!("解析词语JSON失败: {}", e)));
                return "[]".to_string();
            }
        };

        console::log_1(&JsValue::from_str(&format!("词语数量: {}", words.len())));

        if words.is_empty() {
            return "[]".to_string();
        }

        // 找出最大和最小权重
        let max_weight = words
            .iter()
            .map(|w| w.weight)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_weight = words.iter().map(|w| w.weight).fold(f64::INFINITY, f64::min);

        let mut placed_words: Vec<WordPosition> = Vec::new();

        // 计算中心点
        let center_x = self.options.width as f64 / 2.0;
        let center_y = self.options.height as f64 / 2.0;

        for word in words {
            // 计算字体大小
            let size = if max_weight == min_weight {
                self.options.max_size
            } else {
                self.options.min_size
                    + ((word.weight - min_weight) / (max_weight - min_weight))
                        * (self.options.max_size - self.options.min_size)
            };

            // 计算大致的宽度和高度 (这部分在真实实现中需要从canvas获取)
            // 这里我们使用一个粗略估计
            let word_width = size * 0.6 * word.text.len() as f64;
            let word_height = size;

            // 旋转角度
            let rotation = match word.rotate {
                Some(r) => r,
                None if self.options.rotation_range > 0.0 => {
                    (js_sys::Math::random() * 2.0 - 1.0) * self.options.rotation_range
                }
                None => 0.0,
            };

            // 尝试放置单词
            if let Some((x, y)) =
                self.find_position_for_word(center_x, center_y, word_width, word_height, rotation)
            {
                // 标记网格为已占用
                self.mark_grid_as_occupied(x, y, word_width, word_height, rotation);

                // 添加到已放置单词
                placed_words.push(WordPosition {
                    text: word.text,
                    weight: word.weight,
                    x,
                    y,
                    rotate: rotation,
                    color: word.color,
                    size: Some(size),
                });
            }
        }

        // 将结果序列化为JSON
        serde_json::to_string(&placed_words).unwrap_or_else(|_| "[]".to_string())
    }

    // 查找单词的放置位置
    fn find_position_for_word(
        &self,
        center_x: f64,
        center_y: f64,
        word_width: f64,
        word_height: f64,
        rotation: f64,
    ) -> Option<(f64, f64)> {
        // 开始设置螺旋参数
        let mut a = 0.0; // 角度
        let mut step = 0.1; // 螺旋步长
        let dt = match self.options.spiral.as_str() {
            "rectangular" => 2.0,
            _ => step, // archimedean或其他
        };

        let mut t = 0.0; // 螺旋参数

        // 尝试最多1000个位置
        for _attempt in 0..1000 {
            let mut x = center_x;
            let mut y = center_y;

            // 计算螺旋位置
            if self.options.spiral == "archimedean" {
                x += a * Math::cos(t);
                y += a * Math::sin(t);
                a += step;
            } else if self.options.spiral == "rectangular" {
                let sign = |n: f64| -> f64 {
                    if n < 0.0 {
                        -1.0
                    } else {
                        1.0
                    }
                };

                let k = Math::floor(t / dt) as i32;
                if k % 2 == 0 {
                    x += sign(Math::cos(t)) * a;
                    y += sign(Math::sin(t)) * a;
                } else {
                    x += sign(Math::sin(t)) * a;
                    y += sign(Math::cos(t)) * a;
                }
                a += step;
            }

            t += dt;

            // 检查这个位置是否已占用
            if !self.check_collision(x, y, word_width, word_height, rotation) {
                return Some((x, y));
            }
        }

        None
    }

    // 检查碰撞
    fn check_collision(&self, x: f64, y: f64, width: f64, height: f64, rotation: f64) -> bool {
        // 简化的碰撞检测 - 在真实实现中需要更复杂的算法
        // 这里我们检查一个旋转的矩形是否与网格中的任何已占用点重叠

        // 计算旋转后的矩形四个角的坐标
        let sin_rot = rotation.sin();
        let cos_rot = rotation.cos();

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        // 定义矩形的四个角相对于中心的位置
        let corners = [
            (-half_width, -half_height),
            (half_width, -half_height),
            (half_width, half_height),
            (-half_width, half_height),
        ];

        // 旋转并移动到(x,y)位置
        let rotated_corners: Vec<(f64, f64)> = corners
            .iter()
            .map(|(corner_x, corner_y)| {
                let rotated_x = corner_x * cos_rot - corner_y * sin_rot + x;
                let rotated_y = corner_x * sin_rot + corner_y * cos_rot + y;
                (rotated_x, rotated_y)
            })
            .collect();

        // 找出覆盖的网格单元
        let min_x = rotated_corners
            .iter()
            .map(|(x, _)| x)
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_x = rotated_corners
            .iter()
            .map(|(x, _)| x)
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_y = rotated_corners
            .iter()
            .map(|(_, y)| y)
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_y = rotated_corners
            .iter()
            .map(|(_, y)| y)
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        // 转换为网格坐标
        let grid_min_x = (min_x as usize / self.grid_size).max(0);
        let grid_max_x = ((max_x as usize / self.grid_size) + 1).min(self.grid.len() - 1);
        let grid_min_y = (min_y as usize / self.grid_size).max(0);
        let grid_max_y = ((max_y as usize / self.grid_size) + 1).min(self.grid[0].len() - 1);

        // 检查所有覆盖的网格单元是否有碰撞
        for i in grid_min_x..=grid_max_x {
            for j in grid_min_y..=grid_max_y {
                if i < self.grid.len() && j < self.grid[i].len() && self.grid[i][j] {
                    return true; // 碰撞
                }
            }
        }

        // 检查是否超出边界
        if min_x < 0.0
            || max_x > self.options.width as f64
            || min_y < 0.0
            || max_y > self.options.height as f64
        {
            return true; // 边界碰撞
        }

        false // 没有碰撞
    }

    // 标记网格为已占用
    fn mark_grid_as_occupied(&mut self, x: f64, y: f64, width: f64, height: f64, rotation: f64) {
        // 与check_collision类似的逻辑，但是标记为已占用
        let sin_rot = rotation.sin();
        let cos_rot = rotation.cos();

        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let corners = [
            (-half_width, -half_height),
            (half_width, -half_height),
            (half_width, half_height),
            (-half_width, half_height),
        ];

        let rotated_corners: Vec<(f64, f64)> = corners
            .iter()
            .map(|(corner_x, corner_y)| {
                let rotated_x = corner_x * cos_rot - corner_y * sin_rot + x;
                let rotated_y = corner_x * sin_rot + corner_y * cos_rot + y;
                (rotated_x, rotated_y)
            })
            .collect();

        let min_x = rotated_corners
            .iter()
            .map(|(x, _)| x)
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_x = rotated_corners
            .iter()
            .map(|(x, _)| x)
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min_y = rotated_corners
            .iter()
            .map(|(_, y)| y)
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let max_y = rotated_corners
            .iter()
            .map(|(_, y)| y)
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let grid_min_x = (min_x as usize / self.grid_size).max(0);
        let grid_max_x = ((max_x as usize / self.grid_size) + 1).min(self.grid.len() - 1);
        let grid_min_y = (min_y as usize / self.grid_size).max(0);
        let grid_max_y = ((max_y as usize / self.grid_size) + 1).min(self.grid[0].len() - 1);

        // 标记所有覆盖的网格单元为已占用
        for i in grid_min_x..=grid_max_x {
            for j in grid_min_y..=grid_max_y {
                if i < self.grid.len() && j < self.grid[i].len() {
                    self.grid[i][j] = true;
                }
            }
        }
    }
}
