/*

    链接基因: 两个神经细胞、权重、enabled、recurrent、innovation number

    神经细胞基因: 输入/输出/隐藏/偏移、ID


 */
use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq)]
pub enum NeuronType {
    Input,
    Hidden,
    Output,
    Bias,
    None,
}

///神经元基因
#[derive(Clone)]
pub struct NeuronGene{
    //它的标识码
    pub id: i32,
    //它的类型
    pub neuron_type: NeuronType,
    //是否是自反神经细胞
    pub recurrent: bool,
    //设置S形函数的曲率（弯曲性）
    pub activation_response: f64,

    //在网格中的位置(用于在显示器上输出网络)
    //也可用于计算整个网络的深度以及确定一个新加入的链接是否返回链接
    pub split_y: f64,
    pub split_x: f64,
}

impl NeuronGene {
    pub fn new(neuron_type: NeuronType, id: i32, y: f64, x: f64, r: bool) -> NeuronGene {
        NeuronGene {
            id: id,
            neuron_type: neuron_type,
            recurrent: r,
            split_y: y,
            split_x: x,
            activation_response: 1.0,
        }
    }
}

///链接基因
#[derive(Clone)]
pub struct LinkGene{
    //该链接所链接的两个神经细胞标识
    pub from_neuron: i32,
    pub to_neuron: i32,
    pub weight: f64,
    //本link当前是否为Enabled
    pub enabled: bool,
    //指明本link是否为Recurrent的标志
    pub recurrent: bool,
    pub innovation_id: i32,
}

impl LinkGene {
    pub fn new(_in: i32, out: i32, enable: bool, tag: i32, w: f64, rec: bool) -> LinkGene {
        LinkGene {
            from_neuron: _in,
            to_neuron: out,
            weight: w,
            enabled: enable,
            recurrent: rec,
            innovation_id: tag,
        }
    }
}

//cmp
impl Ord for LinkGene {
    fn cmp(&self, other: &LinkGene) -> Ordering {
        self.innovation_id
            .partial_cmp(&other.innovation_id)
            .unwrap()
    }
}
impl Eq for LinkGene {}

//比较运算符需要实现PartialOrd和PartialEq
impl PartialOrd for LinkGene {
    fn partial_cmp(&self, other: &LinkGene) -> Option<Ordering> {
        self.innovation_id.partial_cmp(&other.innovation_id)
    }
}

impl PartialEq for LinkGene {
    fn eq(&self, other: &LinkGene) -> bool {
        self.innovation_id == other.innovation_id
    }
}