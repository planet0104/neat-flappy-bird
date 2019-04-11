
use std::rc::Rc;
use std::cell::RefCell;

use super::genes::*;

#[derive(Clone)]
pub struct Link {
    //指向与本link相连接的两个神经细胞的指针
    _in: Rc<RefCell<Neuron>>,
    out: Rc<RefCell<Neuron>>,

    //连接权重
    weight: f64,

    //这个链接是一个循环链接吗？
    recurrent: bool,
}

impl Link {
    pub fn new(w: f64, _in: Rc<RefCell<Neuron>>, out: Rc<RefCell<Neuron>>, rec: bool) -> Link {
        Link {
            weight: w,
            _in: _in,
            out: out,
            recurrent: rec,
        }
    }
}

pub struct Neuron {
    //所有链接进入这个神经元
    links_in: Vec<Link>,

    //和输出的链接
    links_out: Vec<Link>,

    //权重x输入的和
    sum_activation: f64,

    //这个神经元的输出
    output: f64,

    //这是什么类型的神经元？
    neuron_type: NeuronType,

    //其识别号码
    neuron_id: i32,

    //设置Sigmoid函数的曲率
    activation_response: f64,

    //用于表型的可视化
    pos_x: i32,
    pos_y: i32,
    split_y: f64,
    split_x: f64,
}

impl Neuron {
    pub fn new(tp: NeuronType, id: i32, y: f64, x: f64, act_response: f64) -> Neuron {
        Neuron {
            neuron_type: tp,
            neuron_id: id,
            sum_activation: 0.0,
            output: 0.0,
            pos_x: 0,
            pos_y: 0,
            split_y: y,
            split_x: x,
            activation_response: act_response,
            links_in: vec![],
            links_out: vec![],
        }
    }

    pub fn links_out(&mut self) -> &mut Vec<Link> {
        &mut self.links_out
    }

    pub fn links_in(&mut self) -> &mut Vec<Link> {
        &mut self.links_in
    }
}

pub struct NeuralNet {
    neurons: Vec<Rc<RefCell<Neuron>>>,
    //网络深度
    depth: i32,
}

//更新网络时你需要从snapshot与active两个参数类型中选一个。
//如果选择snapshot(快照方式)，则网络深度值用来控制从输入开始对整个网络刷新。
//如果宣传了active(激活方式)，则网络在每一个时间步骤(time-step)种获得更新
#[derive(Debug, PartialEq)]
pub enum RunType {
    Snapshot,
    Active,
}

//S形响应曲线
//当已知神经细胞所有输入x权重的乘积之和时，这一方法将它送入S形的激励函数
fn sigmoid(netinput: f64, response: f64) -> f64 {
    1.0 / (1.0 + (-netinput / response).exp())
}

impl NeuralNet {
    pub fn empty() -> NeuralNet {
        NeuralNet {
            neurons: vec![],
            depth: 0,
        }
    }
    pub fn new(neurons: Vec<Rc<RefCell<Neuron>>>, depth: i32) -> NeuralNet {
        NeuralNet {
            neurons: neurons,
            depth: depth,
        }
    }

    //更新此时钟周期的网络
    pub fn update(&mut self, inputs: &[f64], run_type: RunType) -> Vec<f64> {
        //创建一个用来存放outputs的Vec
        let mut outputs: Vec<f64> = vec![];
        //如果模式为snapshot(快照模式),则要求所有的神经细胞被重复通过
        //和网络深度一样多的次数。如果模式为active(激活模式),则此方法只要
        //经过一次迭代就可以返回一个输出
        let flush_count = if run_type == RunType::Snapshot {
            self.depth
        } else {
            1
        };
        //对网络重复循环 flush_count 次
        for _ in 0..flush_count {
            //清除输出Vec
            outputs.clear();

            //这是当前神经细胞的一个下标
            let mut neuron = 0;
            //首先将'input'神经元的输出设置为等于传入函数的值
            while self.neurons[neuron].borrow().neuron_type == NeuronType::Input {
                self.neurons[neuron].borrow_mut().output = inputs[neuron];
                neuron += 1;
            }
            //将偏移的输出设置为1
            self.neurons[neuron].borrow_mut().output = 1.0;
            neuron += 1;

            //然后用每次改变一个神经细胞的办法来遍历整个网络
            while neuron < self.neurons.len() {
                //这个sum用来保存所有输入x权重的总和
                let mut sum = 0.0;
                let mut ne = self.neurons[neuron].borrow_mut();
                //通过对进入该神经细胞的所有连接的循环，将该神经细胞各输入值加在一起
                for lnk in &ne.links_in {
                    //得到lnk连接的权重
                    let weight = lnk.weight;
                    //从该链接的进入端神经细胞得到输出
                    let neuron_output = lnk._in.borrow().output;
                    //将次输出加入总和sum中
                    sum += weight * neuron_output;
                }

                //现在让总和输入激励函数，并把其结果赋给这一神经细胞的输出
                let sigmoid_output = sigmoid(sum, ne.activation_response);
                ne.output = sigmoid_output;
                //println!("neuron={}", neuron);
                if ne.neuron_type == NeuronType::Output {
                    //加入到输出
                    outputs.push(ne.output);
                }
                //下一个神经细胞
                neuron += 1;
            }
        } //进入通过网络的下一次迭代
          //如果执行了这种类型的更新,网络输出需要进行复位(reset)，否则由它建立的网络可能会和训练数据的输入顺序有关
        if run_type == RunType::Snapshot {
            for n in &mut self.neurons {
                n.borrow_mut().output = 0.0;
            }
        }
        //返回输出
        outputs
    }

    //在用户指定的一个窗口中回执网络的图形
    // pub fn draw_net(&mut self, surface: ui::Surface, left: i32, top: i32, right: i32, bottom: i32) {
    //     //边框宽度
    //     let border = 10;
    //     //最大线厚度
    //     let max_thickness = 5.0;
    //     tidy_x_splits(&mut self.neurons);
    //     //遍历神经元并分配x / y坐标
    //     let span_x = right - left;
    //     let span_y = top - bottom - (2 * border);
    //     for neuron in &mut self.neurons {
    //         neuron.pos_x = left + (span_x as f64 * neuron.split_x) as i32;
    //         neuron.pos_y = (top - border) - (span_y as f64 * neuron.split_y) as i32;
    //         //println!("神经元类型:{:?} split_x:{}, split_y:{}", n.neuron_type, n.split_x, n.split_y);
    //     }

    //     //创建一些笔和画笔来绘制
    //     let grey_pen = ui::solid_pen(1, ui::rgb(200, 200, 200));
    //     let red_pen = ui::solid_pen(1, ui::rgb(255, 0, 0));
    //     let green_pen = ui::solid_pen(1, ui::rgb(0, 200, 0));

    //     //创建实心画刷
    //     let red_brush = ui::solid_brush(ui::rgb(255, 0, 0));

    //     let old_pen = ui::select_pen(surface, red_pen);
    //     let old_brush = ui::select_hollow_brush(surface);

    //     //神经元的半径
    //     let rad_neuron = span_x as f64 / 60.0;
    //     let rad_link = rad_neuron as f64 * 1.5;

    //     //现在我们有一个X，Y的pos，我们可以得到绘图的每一个神经元。 首先通过网络中的每个神经元绘制链接
    //     for neuron in &self.neurons {
    //         //抓取这个神经元位置作为每个连接的起始位置
    //         let start_x = neuron.pos_x;
    //         let start_y = neuron.pos_y;

    //         //这是一个偏见神经元吗？ 如果是，请将链接绘制成绿色
    //         let bias = neuron.neuron_type == NeuronType::Bias;
    //         //现在遍历每个传出的链接来获取终点
    //         for lnk in &neuron.links_out {
    //             let end_x = self.neurons[lnk.out].pos_x;
    //             let end_y = self.neurons[lnk.out].pos_y;

    //             //如果链接向前画一条直线
    //             if !lnk.recurrent && !bias {
    //                 let mut thickness = lnk.weight.abs() as f32;
    //                 clamp(&mut thickness, 0.0, max_thickness);
    //                 let pen = if lnk.weight <= 0.0 {
    //                     //创建一个用于抑制重量的黄色笔
    //                     ui::solid_pen(thickness as i32, ui::rgb(240, 230, 170))
    //                 } else {
    //                     //灰色或兴奋
    //                     ui::solid_pen(thickness as i32, ui::rgb(200, 200, 200))
    //                 };

    //                 let temp_pen = ui::select_pen(surface, pen);
    //                 //绘制连接
    //                 ui::move_to_ex_i32(surface, start_x, start_y);
    //                 ui::line_to_i32(surface, end_x, end_y);

    //                 ui::select_pen(surface, temp_pen);
    //                 ui::delete_pen(pen);
    //             } else if !lnk.recurrent && bias {
    //                 ui::select_pen(surface, green_pen);
    //                 //绘制连接
    //                 ui::move_to_ex_i32(surface, start_x, start_y);
    //                 ui::line_to_i32(surface, end_x, end_y);
    //             } else {
    //                 //循环链接绘制为红色
    //                 if start_x == end_x && start_y == end_y {
    //                     let mut thickness = lnk.weight.abs() as f32;
    //                     clamp(&mut thickness, 0.0, max_thickness);
    //                     let pen = if lnk.weight <= 0.0 {
    //                         //蓝色为抑制
    //                         ui::solid_pen(thickness as i32, ui::rgb(0, 0, 255))
    //                     } else {
    //                         //红色为兴奋
    //                         ui::solid_pen(thickness as i32, ui::rgb(255, 0, 0))
    //                     };
    //                     let temp_pen = ui::select_pen(surface, pen);
    //                     //我们有一个递归链接到相同的神经元绘制一个椭圆
    //                     let x = neuron.pos_x as f64;
    //                     let y = neuron.pos_y as f64 - (1.5 * rad_neuron);
    //                     ui::ellipse(
    //                         surface,
    //                         (x - rad_link) as i32,
    //                         (y - rad_link) as i32,
    //                         (x + rad_link) as i32,
    //                         (y + rad_link) as i32,
    //                     );
    //                     ui::select_pen(surface, temp_pen);
    //                     ui::delete_pen(pen);
    //                 } else {
    //                     let mut thickness = lnk.weight.abs() as f32;
    //                     clamp(&mut thickness, 0.0, max_thickness);
    //                     let pen = if lnk.weight <= 0.0 {
    //                         //蓝色为抑制
    //                         ui::solid_pen(thickness as i32, ui::rgb(0, 0, 255))
    //                     } else {
    //                         //红色为兴奋
    //                         ui::solid_pen(thickness as i32, ui::rgb(255, 0, 0))
    //                     };
    //                     let temp_pen = ui::select_pen(surface, pen);
    //                     //绘制连接
    //                     ui::move_to_ex_i32(surface, start_x, start_y);
    //                     ui::line_to_i32(surface, end_x, end_y);

    //                     ui::select_pen(surface, temp_pen);
    //                     ui::delete_pen(pen);
    //                 }
    //             }
    //         }
    //     }

    //     //现在绘制神经元及其ID
    //     ui::select_brush(surface, red_brush);
    //     ui::select_black_pen(surface);

    //     for neuron in &self.neurons {
    //         let x = neuron.pos_x as f64;
    //         let y = neuron.pos_y as f64;
    //         ui::ellipse(
    //             surface,
    //             (x - rad_neuron) as i32,
    //             (y - rad_neuron) as i32,
    //             (x + rad_neuron) as i32,
    //             (y + rad_neuron) as i32,
    //         );
    //     }

    //     //清理
    //     ui::select_pen(surface, old_pen);
    //     ui::select_brush(surface, old_brush);

    //     ui::delete_pen(red_pen);
    //     ui::delete_pen(grey_pen);
    //     ui::delete_pen(green_pen);
    //     ui::delete_pen(old_pen);
    //     ui::delete_brush(red_brush);
    //     ui::delete_brush(old_brush);
    // }
}

//这是一个修复，以防止显示时神经元重叠
fn tidy_x_splits(neurons: &mut Vec<Neuron>) {
    //存储具有相同splitY值的任何神经元的索引
    let mut same_level_neurons: Vec<usize> = vec![];
    //存储已经检查的所有splitY值
    let mut depths_checked: Vec<f64> = vec![];
    //为每个神经元找到所有相同Split级别的神经元
    for n in 0..neurons.len() {
        let this_depth = neurons[n].split_y;
        //检查我们是否已经在这个深度调整了神经元
        let mut already_checked = false;
        for i in 0..depths_checked.len() {
            if depths_checked[i] == this_depth {
                already_checked = true;
                break;
            }
        }

        //将此深度添加到检查的深度。
        depths_checked.push(this_depth);
        //如果这个深度还没有被调整
        if !already_checked {
            //清除此存储并添加我们正在检查的神经元索引
            same_level_neurons.clear();
            same_level_neurons.push(n);

            //找到这个splitY深度的所有神经元
            for i in (n + 1)..neurons.len() {
                if neurons[i].split_y == this_depth {
                    //将索引添加到这个神经元
                    same_level_neurons.push(i);
                }
            }

            //计算每个神经元之间的距离
            let slice = 1.0 / (same_level_neurons.len() as f64 + 1.0);

            //将这个级别的所有神经元分开
            for i in 0..same_level_neurons.len() {
                let idx = same_level_neurons[i];
                neurons[idx].split_x = (i as f64 + 1.0) * slice;
            }
        }
    } //下一个要检查的神经元
}
