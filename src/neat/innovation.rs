//处理实施Kenneth Owen Stanley和Risto Miikkulainen的NEAT想法时使用的基因组创新的类。

use super::genes::*;

#[derive(PartialEq)]
pub enum InnovationType {
    NewNeuron,
    NewLink,
}

pub struct Innovation {
    //新类型还是新连接?
    innovation_type: InnovationType,
    innovation_id: i32,
    neuron_in: i32,
    neuron_out: i32,
    neuron_id: i32,
    neuron_type: NeuronType,
    //如果创新是神经元，我们需要保留记录
    //在树中的位置（用于展示）
    split_x: f64,
    split_y: f64,
}

impl Innovation {
    pub fn new(_in: i32, out: i32, t: InnovationType, inov_id: i32) -> Innovation {
        Innovation {
            neuron_in: _in,
            neuron_out: out,
            innovation_type: t,
            innovation_id: inov_id,
            neuron_id: 0,
            split_x: 0.0,
            split_y: 0.0,
            neuron_type: NeuronType::None,
        }
    }

    pub fn with_split(
        _in: i32,
        out: i32,
        t: InnovationType,
        inov_id: i32,
        neuron_type: NeuronType,
        x: f64,
        y: f64,
    ) -> Innovation {
        Innovation {
            neuron_in: _in,
            neuron_out: out,
            innovation_type: t,
            innovation_id: inov_id,
            neuron_id: 0,
            neuron_type: neuron_type,
            split_x: x,
            split_y: y,
        }
    }

    pub fn from(neuron: &NeuronGene, innov_id: i32, neuron_id: i32) -> Innovation {
        Innovation {
            innovation_id: innov_id,
            innovation_type: InnovationType::NewNeuron,
            neuron_id: neuron_id,
            split_x: neuron.split_x,
            split_y: neuron.split_y,
            neuron_type: neuron.neuron_type,
            neuron_in: -1,
            neuron_out: -1,
        }
    }
}

pub struct Innovations {
    innovs: Vec<Innovation>,
    next_neuron_id: i32,
    next_innovation_num: i32,
    /* 构造函数和无关的成员均已略去 */
}

impl Innovations {
    //给出一系列起始基因和起始神经元，这个构造函数添加了所有适当的创新。
    pub fn new(start_genes: &[LinkGene], start_neurons: &[NeuronGene]) -> Innovations {
        let mut innovation = Innovations {
            next_neuron_id: 0,
            next_innovation_num: 0,
            innovs: vec![],
        };
        innovation.new_ctor(start_genes, start_neurons);
        innovation
    }

    //给出一系列起始基因和起始神经元，这个构造函数添加了所有适当的创新。
    fn new_ctor(&mut self, start_genes: &[LinkGene], start_neurons: &[NeuronGene]) {
        //添加神经元
        for nd in start_neurons {
            self.innovs.push(Innovation::from(
                nd,
                self.next_innovation_num,
                self.next_neuron_id,
            ));
            self.next_innovation_num += 1;
            self.next_neuron_id += 1;
        }
        //添加连接
        for gen in start_genes {
            let new_innov = Innovation::new(
                gen.from_neuron,
                gen.to_neuron,
                InnovationType::NewLink,
                self.next_innovation_num,
            );
            self.innovs.push(new_innov);
            self.next_innovation_num += 1;
        }
    }

    //检查这个创新是否已经发生。 如果有的话
    //返回创新ID。 如果不是，则返回一个负值。
    pub fn check_innovation(&self, _in: i32, out: i32, iv_type: InnovationType) -> i32 {
        //如果没有匹配返回一个负值
        let mut innovation_id = -1;
        for inv in &self.innovs {
            if inv.neuron_in == _in && inv.neuron_out == out && inv.innovation_type == iv_type {
                //找到一个匹配，所以将这个创新号码分配给id
                innovation_id = inv.innovation_id;
                break;
            }
        }
        innovation_id
    }

    //创造一个新的创新，并返回其ID
    pub fn create_new_innovation(&mut self, _in: i32, out: i32, iv_type: InnovationType) -> i32 {
        let new_neuron = iv_type == InnovationType::NewNeuron;
        let mut new_innov = Innovation::new(_in, out, iv_type, self.next_innovation_num);
        if new_neuron {
            new_innov.neuron_id = self.next_neuron_id;
            self.next_neuron_id += 1;
        }
        self.innovs.push(new_innov);
        self.next_innovation_num += 1;
        self.next_neuron_id - 1
    }

    pub fn create_new_innovation_with_pos(
        &mut self,
        from: i32,
        to: i32,
        iv_type: InnovationType,
        neuron_type: NeuronType,
        x: f64,
        y: f64,
    ) -> i32 {
        let new_neuron = iv_type == InnovationType::NewNeuron;
        let mut new_innov = Innovation::with_split(
            from,
            to,
            iv_type,
            self.next_innovation_num,
            neuron_type,
            x,
            y,
        );
        if new_neuron {
            new_innov.neuron_id = self.next_neuron_id;
            self.next_neuron_id += 1;
        }
        self.innovs.push(new_innov);
        self.next_innovation_num += 1;
        self.next_neuron_id - 1
    }

    pub fn create_neuron_from_id(&self, neuron_id: i32) -> NeuronGene {
        let mut temp = NeuronGene::new(NeuronType::Hidden, 0, 0.0, 0.0, false);
        for inv in &self.innovs {
            if inv.neuron_id == neuron_id {
                temp.neuron_type = inv.neuron_type;
                temp.id = inv.neuron_id;
                temp.split_x = inv.split_x;
                temp.split_y = inv.split_y;
                break;
            }
        }
        temp
    }

    pub fn next_number(&mut self) -> i32 {
        //self.next_innovation_num += 1;
        self.next_innovation_num
    }

    pub fn get_neuron_id(&self, inv: i32) -> i32 {
        self.innovs[inv as usize].neuron_id
    }

    pub fn flush(&mut self) {
        self.innovs.clear();
    }
}

