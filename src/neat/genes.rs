use crate::neat::phenotype::{Link, NeuralNet, Neuron};
use crate::neat::utils::{random_clamped_f64, random_float, random_int, random_usize, sqrt_usize};
use std::cmp::Ordering;
//-------------------------------------------------------//
//---链接基因---------------------------------------------//
//-------------------------------------------------------//
#[derive(Clone)]
pub struct LinkGene {
    //链接的两个神经细胞标识
    from_neuron: i32,
    to_neuron: i32,
    weight: f64,
    //本link当前是否为Enabled
    enabled: bool,
    //指明本link是否为Recurrent的标志
    recurrent: bool,

    innovation_id: i32,
}

impl LinkGene {
    pub fn temp() -> LinkGene {
        LinkGene {
            from_neuron: 0,
            to_neuron: 0,
            weight: 0.0,
            enabled: false,
            recurrent: false,
            innovation_id: -1,
        }
    }
    pub fn new(_in: i32, out: i32, enable: bool, tag: i32, w: f64) -> LinkGene {
        LinkGene {
            from_neuron: _in,
            to_neuron: out,
            weight: w,
            enabled: enable,
            recurrent: false,
            innovation_id: tag,
        }
    }

    pub fn create(_in: i32, out: i32, enable: bool, tag: i32, w: f64, rec: bool) -> LinkGene {
        LinkGene {
            from_neuron: _in,
            to_neuron: out,
            weight: w,
            enabled: enable,
            recurrent: rec,
            innovation_id: tag,
        }
    }

    pub fn innovation_id(&self) -> i32 {
        self.innovation_id
    }

    pub fn from_neuron(&self) -> i32 {
        self.from_neuron
    }
    pub fn to_neuron(&self) -> i32 {
        self.to_neuron
    }
}

impl Ord for LinkGene {
    fn cmp(&self, other: &LinkGene) -> Ordering {
        self.innovation_id
            .partial_cmp(&other.innovation_id)
            .unwrap()
    }
}

//比较运算符重载
impl PartialOrd for LinkGene {
    fn partial_cmp(&self, other: &LinkGene) -> Option<Ordering> {
        self.innovation_id.partial_cmp(&other.innovation_id)
    }
}
//等于运算符重载
impl PartialEq for LinkGene {
    fn eq(&self, other: &LinkGene) -> bool {
        self.innovation_id == other.innovation_id
    }
}
impl Eq for LinkGene {}

//-------------------------------------------------------//
//-------------------------------------------------------//

#[derive(Debug, PartialEq, Clone)]
pub enum NeuronType {
    Input,
    Hidden,
    Output,
    Bias,
    None,
}

//-------------------------------------------------------//
//---神经细胞---------------------------------------------//
//-------------------------------------------------------//
#[derive(Clone)]
pub struct NeuronGene {
    //它的标识码
    id: i32,
    //它的类型
    neuron_type: NeuronType,
    //是否是自反神经细胞
    recurrent: bool,
    //设置S形函数的曲率（弯曲性）
    activation_response: f64,
    //在网格中的位置(用于在显示器上输出网络)
    //也可用于计算整个网络的深度以及确定一个新加入的链接是否返回链接
    split_y: f32,
    split_x: f32,
}

impl Drop for NeuronGene {
    fn drop(&mut self) {
        //println!("NeuronGene::Drop..");
    }
}

impl NeuronGene {
    pub fn new(neuron_type: NeuronType, id: i32, y: f32, x: f32, r: bool) -> NeuronGene {
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

//-------------------------------------------------------//
//-------------------------------------------------------//
#[derive(Debug, PartialEq)]
pub enum InnovationType {
    NewNeuron,
    NewLink,
}

pub struct SInnovation {
    //新类型还是新连接?
    innovation_type: InnovationType,
    innovation_id: i32,
    neuron_in: i32,
    neuron_out: i32,
    neuron_id: i32,
    neuron_type: NeuronType,
    //如果创新是神经元，我们需要保留记录
    //在树中的位置（用于展示）
    split_x: f32,
    split_y: f32,
}

impl Drop for SInnovation {
    fn drop(&mut self) {
        //println!("SInnovation::Drop..");
    }
}

impl SInnovation {
    pub fn new(_in: i32, out: i32, t: InnovationType, inov_id: i32) -> SInnovation {
        SInnovation {
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
        x: f32,
        y: f32,
    ) -> SInnovation {
        SInnovation {
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

    pub fn from(neuron: &NeuronGene, innov_id: i32, neuron_id: i32) -> SInnovation {
        SInnovation {
            innovation_id: innov_id,
            innovation_type: InnovationType::NewNeuron,
            neuron_id: neuron_id,
            split_x: neuron.split_x,
            split_y: neuron.split_y,
            neuron_type: neuron.neuron_type.clone(),
            neuron_in: -1,
            neuron_out: -1,
        }
    }
}

pub struct Innovation {
    innovs: Vec<SInnovation>,
    next_neuron_id: i32,
    next_innovation_num: i32,
    /* 构造函数和无关的成员均已略去 */
}

impl Innovation {
    //给出一系列起始基因和起始神经元，这个构造函数添加了所有适当的创新。
    pub fn new(start_genes: &Vec<LinkGene>, start_neurons: &Vec<NeuronGene>) -> Innovation {
        let mut innovation = Innovation {
            next_neuron_id: 0,
            next_innovation_num: 0,
            innovs: vec![],
        };
        innovation.new_ctor(start_genes, start_neurons);
        innovation
    }

    //给出一系列起始基因和起始神经元，这个构造函数添加了所有适当的创新。
    fn new_ctor(&mut self, start_genes: &Vec<LinkGene>, start_neurons: &Vec<NeuronGene>) {
        //添加神经元
        for nd in start_neurons {
            self.innovs.push(SInnovation::from(
                nd,
                self.next_innovation_num,
                self.next_neuron_id,
            ));
            self.next_innovation_num += 1;
            self.next_neuron_id += 1;
        }
        //添加连接
        for gen in start_genes {
            let new_innov = SInnovation::new(
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
        let mut new_innov = SInnovation::new(_in, out, iv_type, self.next_innovation_num);
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
        x: f32,
        y: f32,
    ) -> i32 {
        let new_neuron = iv_type == InnovationType::NewNeuron;
        let mut new_innov = SInnovation::with_split(
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
                temp.neuron_type = inv.neuron_type.clone();
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

//-------------------------------------------------------//
//---基因组-----------------------------------------------//
//-------------------------------------------------------//
#[derive(Clone)]
pub struct Genome {
    //它的标识
    genome_id: i32,
    //组成此基因组所有的神经细胞
    neurons: Vec<NeuronGene>,
    //所有的链接
    links: Vec<LinkGene>,
    //指向它的表现型指针
    phenotype: NeuralNet,
    //它的原始适应性分数
    fitness: f64,
    //它的适应分成绩被放入物种后进行调整
    adjusted_fitness: f64,
    //要求孵化的下一代的子孙数目
    amount_to_spawn: f64,
    //分别用来保存输入和输出数目的两个记录
    num_inputs: usize,
    num_outputs: usize,
    //保存该基因组进入的物种的轨迹(仅用于显示目的)
    species: i32,
}

impl Genome {
    pub fn new() -> Genome {
        Genome {
            genome_id: 0,
            neurons: vec![],
            links: vec![],
            fitness: 0.0,
            amount_to_spawn: 0.0,
            num_inputs: 0,
            num_outputs: 0,
            species: 0,
            adjusted_fitness: 0.0,
            phenotype: NeuralNet::empty(),
        }
    }

    //这个构造函数创建一个最小的基因组，其中有输出+输入神经元，每个输入神经元连接到每个输出神经元。
    pub fn minimal(id: i32, inputs: usize, outputs: usize) -> Genome {
        let mut gen = Genome::new();
        gen.genome_id = id;
        gen.num_inputs = inputs;
        gen.num_outputs = outputs;

        //println!("minimal: inputs={}, outputs={}", inputs, outputs);

        //创建输入神经元
        let input_row_slice = 1.0 / (inputs as f32 + 2.0);
        for i in 0..inputs {
            gen.neurons.push(NeuronGene::new(
                NeuronType::Input,
                i as i32,
                0.0,
                (i as f32 + 2.0) * input_row_slice,
                false,
            ));
        }
        //创建偏移
        gen.neurons.push(NeuronGene::new(
            NeuronType::Bias,
            inputs as i32,
            0.0,
            input_row_slice,
            false,
        ));

        //创建输出神经元
        let output_row_slice = 1.0 / (outputs as f32 + 1.0);
        for i in 0..outputs {
            gen.neurons.push(NeuronGene::new(
                NeuronType::Output,
                (i + inputs + 1) as i32,
                1.0,
                (i as f32 + 1.0) * output_row_slice,
                false,
            ));
        }

        //创建链接基因，将每个输入神经元连接到每个输出神经元
        //分配一个随机权重-1 <w <1
        let mut num_genes = 0;

        for i in 0..inputs + 1 {
            for j in 0..outputs {
                gen.links.push(LinkGene::new(
                    gen.neurons[i].id,
                    gen.neurons[inputs + j + 1].id,
                    true,
                    (inputs + outputs + 1 + num_genes) as i32,
                    random_clamped_f64(),
                ));
                num_genes += 1;
            }
        }

        gen
    }

    //这个构造函数从LinkGenes的Vec创建一个基因组，一个NeuronGenes的载体和一个ID号
    pub fn from(
        id: i32,
        neurons: Vec<NeuronGene>,
        genes: Vec<LinkGene>,
        inputs: usize,
        outputs: usize,
    ) -> Genome {
        Genome {
            genome_id: id,
            neurons: neurons,
            links: genes,
            fitness: 0.0,
            amount_to_spawn: 0.0,
            num_inputs: inputs,
            num_outputs: outputs,
            species: 0,
            adjusted_fitness: 0.0,
            phenotype: NeuralNet::empty(),
        }
    }

    //复制
    // pub fn from(g:&Genome) ->Genome{

    // }

    pub fn split_y(&self, val: i32) -> f64 {
        self.neurons[val as usize].split_y as f64
    }

    //=号重载
    //由基因组创建神经网络
    pub fn create_phenotype(&mut self, depth: i32) {
        //首先应确保删除该基因组原来存在得表现型
        self.delete_phenotype();

        //用于保存表现型所要求的所有神经细胞
        let mut neurons: Vec<Neuron> = vec![];
        //创建所有要求的神经细胞
        //这里有可能neurons是反向的，即input在最后，这种情况要反过来创建，确保Input类型的基因在前边
        if self.neurons[0].neuron_type == NeuronType::Input {
            for n in &self.neurons {
                neurons.push(Neuron::new(
                    n.neuron_type.clone(),
                    n.id,
                    n.split_y as f64,
                    n.split_x as f64,
                    n.activation_response,
                ));
            }
        } else {
            for n in 0..self.neurons.len() {
                neurons.push(Neuron::new(
                    self.neurons[self.neurons.len() - 1 - n].neuron_type.clone(),
                    self.neurons[self.neurons.len() - 1 - n].id,
                    self.neurons[self.neurons.len() - 1 - n].split_y as f64,
                    self.neurons[self.neurons.len() - 1 - n].split_x as f64,
                    self.neurons[self.neurons.len() - 1 - n].activation_response,
                ));
            }
        }

        //println!("neurons.len={}", neurons.len());

        //再创建链接
        for lnk in &self.links {
            //在链接创建之前，要保证链接基因已被打开
            if lnk.enabled {
                //产生指向有关的各个神经细胞的指针
                let from_neuron_pos = self.get_element_pos(lnk.from_neuron);
                let to_neuron_pos = self.get_element_pos(lnk.to_neuron);

                //在这两个神经细胞之间创建一个链接，并为存入的基因分配权重
                let link = Link::new(lnk.weight, from_neuron_pos, to_neuron_pos, lnk.recurrent);

                //把新的链接加入到神经细胞
                neurons[from_neuron_pos].links_out().push(link.clone());
                neurons[to_neuron_pos].links_in().push(link);
            }
        }

        //每个神经细胞都已经包含了所有的链接信息，然后利用他们创建一个神经网络
        self.phenotype = NeuralNet::new(neurons, depth);
    }

    pub fn phenotype(&mut self) -> &mut NeuralNet {
        &mut self.phenotype
    }

    //删除神经网络
    pub fn delete_phenotype(&mut self) {
        //Drop
        self.phenotype = NeuralNet::empty();
    }

    //按突变率在基因组中添加一个连接
    //本算子加入下列3种不同连接中的一种
    // 正向连接(forward link)
    // 返回连接(recurrent link)
    // 自环的返回连接(looped recurrent link)
    pub fn add_link(
        &mut self,
        mutation_rate: f32,
        chance_of_lopped: f32,
        innovation: &mut Innovation,
        mut num_trys_to_find_loop: i32,
        mut num_trys_to_add_link: i32,
    ) {
        //根据突变率来确定是否立即返回
        if random_float() > mutation_rate {
            return;
        }

        //确定要连接的两个神经细胞的持有者。如果要连接的是两个有效的神经细胞这些值将变成 >= 0
        let mut id_neuron1 = -1;
        let mut id_neuron2 = -1;
        //如果选择加入循环连接，则次标志为true
        let mut recurrent = false;
        //首先检查需要创建的连接是否返回到同一个神经细胞本身
        if random_float() < chance_of_lopped {
            //如果是，则重复实验 num_trys_to_find_loop次,寻找一个既不是输入也不是偏移
            //且没有一个环形自反连接的神经细胞
            while num_trys_to_find_loop > 0 {
                num_trys_to_find_loop -= 1;
                //任取一个神经细胞
                let neuron_pos = random_usize(self.num_inputs + 1, self.neurons.len() - 1);

                //做检查以确保神经细胞没有自反连接，也不是一个输入或偏移神经细胞
                if !self.neurons[neuron_pos].recurrent {
                    match self.neurons[neuron_pos].neuron_type {
                        NeuronType::Bias | NeuronType::Input => (),
                        _ => {
                            id_neuron1 = self.neurons[neuron_pos].id;
                            id_neuron2 = self.neurons[neuron_pos].id;
                            self.neurons[neuron_pos].recurrent = true;
                            recurrent = true;
                            num_trys_to_find_loop = 0;
                        }
                    }
                }
            }
        } else {
            //如果为否: 试着去寻找两个不连接的神经细胞。一共要尝试num_trys_to_add_link次
            while num_trys_to_add_link > 0 {
                num_trys_to_add_link -= 1;
                //选择两个神经细胞, 第二个不能是输入或偏移神经细胞
                id_neuron1 = self.neurons[random_int(0, self.neurons.len() as i32 - 1) as usize].id;
                id_neuron2 = self.neurons[random_int(
                    self.num_inputs as i32 + 1,
                    self.neurons.len() as i32 - 1,
                ) as usize]
                    .id;
                if id_neuron2 == 2 {
                    //????
                    continue;
                }
                //保证这两个神经细胞没有连接，且也不是同一个神经细胞
                if !(self.duplicate_link(id_neuron1, id_neuron2) || id_neuron1 == id_neuron2) {
                    num_trys_to_add_link = 0;
                } else {
                    id_neuron1 = -1;
                    id_neuron2 = -1;
                }
            }
        }

        //如果寻找连接不成功，则立刻返回
        if id_neuron1 < 0 || id_neuron1 < 0 {
            return;
        }

        //检查这以创新是否已经创建过了
        let id = innovation.check_innovation(id_neuron1, id_neuron2, InnovationType::NewLink);
        //此连接是返回的吗?
        if self.neurons[self.get_element_pos(id_neuron1)].split_y
            > self.neurons[self.get_element_pos(id_neuron2)].split_y
        {
            recurrent = true;
        }
        if id < 0 {
            //需要创建一个新的创新
            innovation.create_new_innovation(id_neuron1, id_neuron2, InnovationType::NewLink);
            //创建新的基因
            let id = innovation.next_number() - 1;
            let new_gene = LinkGene::create(
                id_neuron1,
                id_neuron1,
                true,
                id,
                random_clamped_f64(),
                recurrent,
            );
            self.links.push(new_gene);
        } else {
            //次创新已存在，下面要做的就是使用已存在的创新标识来创建新基因
            let new_gene = LinkGene::create(
                id_neuron1,
                id_neuron2,
                true,
                id,
                random_clamped_f64(),
                recurrent,
            );
            self.links.push(new_gene);
        }
    }

    //增加一个神经细胞
    pub fn add_neuron(
        &mut self,
        mutation_rate: f32,
        innovations: &mut Innovation,
        mut num_trys_to_find_old_link: i32,
    ) {
        //根据突变率来确定是否返回
        if random_float() > mutation_rate {
            return;
        }
        //如果找到了要插入的新神经细胞的有效连接，则此值将设置为true
        let mut done = false;
        //这将用来保存所选连接基因在links中的索引
        let mut chosen_link = 0usize;
        //首先选择一个进行断裂的连接。如果基因很小，则代码必须对原有的旧连接实行断裂，以保证不出现一连串的链条连接。
        //这里规定，如果基因组包含隐藏神经细胞少于5个，就认为它是太小了，就不能在num_genes-1个连接中随机选择，必须采取其他选择法
        let size_threshold = self.num_inputs + self.num_outputs + 5;
        if self.links.len() < size_threshold {
            while num_trys_to_find_old_link > 0 {
                num_trys_to_find_old_link -= 1;
                //在基因组中选择一个相对于原有连接有偏移的较早的连接
                chosen_link = random_int(
                    0,
                    self.num_genes() as i32 - 1 - sqrt_usize(&self.num_genes()) as i32,
                ) as usize;
                //保证该连接已被enabled并且它不是一个返回连接或带有偏移输入
                let from_neuron = self.links[chosen_link].from_neuron;
                if self.links[chosen_link].enabled
                    && !self.links[chosen_link].recurrent
                    && self.neurons[self.get_element_pos(from_neuron)].neuron_type
                        != NeuronType::Bias
                {
                    done = true;
                    num_trys_to_find_old_link = 0;
                }
            }
            if !done {
                //寻找下一个连接的工作失败
                return;
            }
        } else {
            //基因组具有足够尺寸去接受任何连接
            while !done {
                chosen_link = random_usize(0, self.num_genes() - 1);
                //保证该连接已被enabled并且它不是一个返回连接或带有偏移输入
                let from_neuron = self.links[chosen_link].from_neuron;
                if self.links[chosen_link].enabled
                    && !self.links[chosen_link].recurrent
                    && self.neurons[self.get_element_pos(from_neuron)].neuron_type
                        != NeuronType::Bias
                {
                    done = true;
                }
            }
        }

        //到此，连接已选中，下一步进行神经细胞的插入。首先禁止掉该连接基因
        self.links[chosen_link].enabled = false;
        //再从该基因取得权重(用它作为新加入的一个连接的权重，这样可使连接的断裂不至于扰乱神经网络已经学习得到的东西)
        let original_weight = self.links[chosen_link].weight;
        //标识这个连接所连接的两个神经细胞
        let from = self.links[chosen_link].from_neuron;
        let to = self.links[chosen_link].to_neuron;
        //计算新的神经细胞的深度和宽度，利用深度来确定连接的向前或向后
        let new_depth = (self.neurons[self.get_element_pos(from)].split_y
            + self.neurons[self.get_element_pos(to)].split_y)
            / 2.0;
        let new_width = (self.neurons[self.get_element_pos(from)].split_x
            + self.neurons[self.get_element_pos(to)].split_x)
            / 2.0;
        //检查这一创新是否以前已被群体中其他成员创建过
        let mut id = innovations.check_innovation(from, to, InnovationType::NewNeuron);

        /*  NEAT 可能重复做的事情有下列几种:
            1.寻找一个link。这里假设选择的是link1到link5中的一个
            2.禁止这个link
            3.增加一个新的神经细胞和两个新的link
            4.如果后一个基因组也有同样的link但没有被禁止的话，由第2步禁止的link有可能在此基因组与另一个基因组重组时被重新启用。
                因此，下列的代码用来检查一个神经细胞标识号是否已经在使用。如果是，则函数要为神经细胞创建一个新的创新
        */
        if id >= 0 {
            let neuron_id = innovations.get_neuron_id(id);
            if self.alerady_have_this_neuron_id(neuron_id) {
                id = -1;
            }
        }
        if id < 0 {
            //这是一个新的创新
            //为新的神经细胞加入创新
            let new_neuron_id = innovations.create_new_innovation_with_pos(
                from,
                to,
                InnovationType::NewNeuron,
                NeuronType::Hidden,
                new_width,
                new_depth,
            );
            //创建新的神经细胞基因并将它加入基因组
            self.neurons.push(NeuronGene::new(
                NeuronType::Hidden,
                new_neuron_id,
                new_depth,
                new_width,
                false,
            ));
            //需要两个新的连接创新。当基因断裂而创建两个新连接时，每一个新的连接都需要一个连接创新.

            //---------------------------------------------第一个连接
            //产生下一个创新标识号
            let id_link1 = innovations.next_number();
            //创建新的创新
            innovations.create_new_innovation(from, new_neuron_id, InnovationType::NewLink);
            //创建新的基因
            let link1 = LinkGene::new(from, new_neuron_id, true, id_link1, 1.0);
            self.links.push(link1);

            //---------------------------------------------第一个连接
            //产生下一个创新标识号
            let id_link2 = innovations.next_number();
            //创建新的创新
            innovations.create_new_innovation(new_neuron_id, to, InnovationType::NewLink);
            //创建新的基因
            let link2 = LinkGene::new(new_neuron_id, to, true, id_link2, original_weight);
            self.links.push(link2);
        } else {
            //存在着的创新
            //因该创新已经建立，故可从创新数据库得到相关的神经细胞和连接信息
            let new_neuron_id = innovations.get_neuron_id(id);
            //为两个新连接基因生成创新标识号
            let id_link1 =
                innovations.check_innovation(from, new_neuron_id, InnovationType::NewLink);
            let id_link2 = innovations.check_innovation(new_neuron_id, to, InnovationType::NewLink);
            //下面的情况应该永远不会发生，因为创新*应该*已经出现
            if id_link1 < 0 || id_link2 < 0 {
                println!("Error in Genome::AddNode!");
                return;
            }
            //创建两个新基因来代表新的连接
            let link1 = LinkGene::new(from, new_neuron_id, true, id_link1, 1.0);
            let link2 = LinkGene::new(new_neuron_id, to, true, id_link2, original_weight);
            self.links.push(link1);
            self.links.push(link2);
            //创建新的神经细胞
            let new_neuron = NeuronGene::new(
                NeuronType::Hidden,
                new_neuron_id,
                new_depth,
                new_width,
                false,
            );
            //并将它加入基因组
            self.neurons.push(new_neuron);
        }
    }

    //  对连接权重实行突变
    //  通过基因迭代，并给出一个概率mut_rate的权重。
    //  prob_new_mut是重量可能被全新的重量所取代的机会。
    //  max_pertubation是要应用的最大扰动。
    //  type是我们使用的随机数算法的类型
    pub fn mutate_weights(&mut self, mut_rate: f32, pro_new_mut: f32, max_pertubation: f64) {
        for gen in &mut self.links {
            //我们突变这个基因吗？
            if random_float() < mut_rate {
                //我们将权重改为全新的权重吗？
                if random_float() < pro_new_mut {
                    //使用'type'定义的随机分布来改变权重
                    gen.weight = random_clamped_f64();
                } else {
                    //扰乱权重
                    gen.weight += random_clamped_f64() * max_pertubation;
                }
            }
        }
    }

    //干扰神经细胞的激励响应
    pub fn mutate_activation_response(&mut self, mut_rate: f32, max_pertubation: f64) {
        for gen in &mut self.neurons {
            if random_float() < mut_rate {
                gen.activation_response += random_clamped_f64() * max_pertubation;
            }
        }
    }

    //计算基本基因组和其他基因组之间的兼容性分
    pub fn get_compatibility_score(&self, genome: &Genome) -> f64 {
        //通过逐步减少每个基因组的长度来计算脱落基因、过量基因和匹配基因的数目
        let mut num_disjoint = 0.0;
        let mut num_excess = 0.0;
        let mut num_matched = 0.0;
        //它记录了匹配的基因中权重差的总和
        let mut weight_difference = 0.0;
        //指向每个基因，当一步步减少基因组长度时，它们是递增的
        let (mut g1, mut g2) = (0, 0);
        let link_len = self.links.len() as i32;
        while (g1 < link_len - 1) || (g2 < link_len - 1) {
            //已经到达genome1的结尾处, 但还没有到达genome2的结尾,所以应递增过量的分数
            if g1 == self.links.len() as i32 - 1 {
                g2 += 1;
                num_excess += 1.0;
                continue;
            }
            //反之亦然
            if g2 == genome.links.len() as i32 - 1 {
                g1 += 1;
                num_excess += 1.0;
                continue;
            }
            //获得每一个基因此时的创新标识号
            let id1 = self.links[g1 as usize].innovation_id;
            let id2 = genome.links[g2 as usize].innovation_id;
            //如果创新号相同,则增加匹配分数
            if id1 == id2 {
                g1 += 1;
                g2 += 1;
                num_matched += 1.0;
                //得到这两个基因之间的权重差
                weight_difference +=
                    (self.links[g1 as usize].weight - genome.links[g2 as usize].weight).abs();
            }

            //如果创新号不同，则应增加脱落基因的分数
            if id1 < id2 {
                num_disjoint += 1.0;
                g1 += 1;
            }
            if id1 > id2 {
                num_disjoint += 1.0;
                g2 += 1;
            }
        } //while结束
          //得到最长的基因组的长度
        let mut longest = genome.num_genes();
        if self.num_genes() > longest {
            longest = self.num_genes();
        }
        //下面是应与最终分相乘的系数
        let disjoint = 1.0;
        let excess = 1.0;
        let matched = 0.4;
        //最后计算总分
        excess * num_excess / longest as f64
            + disjoint * num_disjoint / longest as f64
            + matched * weight_difference / num_matched
    }

    pub fn amount_to_spawn(&self) -> f64 {
        self.amount_to_spawn
    }

    //确实如此
    pub fn sort_genes(&mut self) {
        //self.links.sort();
        self.links.sort_by(|a, b| b.cmp(a)); //从大到小排序
    }

    pub fn genes(&self) -> &Vec<LinkGene> {
        &self.links
    }

    pub fn neurons(&self) -> &Vec<NeuronGene> {
        &self.neurons
    }

    pub fn num_genes(&self) -> usize {
        self.links.len()
    }

    pub fn fitness(&self) -> f64 {
        self.fitness
    }

    pub fn set_fitness(&mut self, score: f64) {
        self.fitness = score;
    }

    pub fn num_inputs(&self) -> usize {
        self.num_inputs
    }

    pub fn num_neurons(&self) -> i32 {
        self.neurons.len() as i32
    }

    pub fn num_outputs(&self) -> usize {
        self.num_outputs
    }

    pub fn set_adj_fitness(&mut self, num: f64) {
        self.adjusted_fitness = num;
    }

    pub fn get_adj_fitness(&self) -> f64 {
        self.adjusted_fitness
    }

    pub fn set_amount_to_spawn(&mut self, num: f64) {
        self.amount_to_spawn = num;
    }

    pub fn id(&self) -> i32 {
        self.genome_id
    }

    pub fn set_id(&mut self, id: i32) {
        self.genome_id = id;
    }

    pub fn set_species(&mut self, species: i32) {
        self.species = species;
    }

    pub fn get_species(&self) -> i32 {
        self.species
    }

    //如果指定的链接已是基因组的一个部分, 返回true
    fn duplicate_link(&self, neuron_in: i32, neuron_out: i32) -> bool {
        for gene in &self.links {
            if gene.from_neuron == neuron_in && gene.to_neuron == neuron_out {
                //已经有了此连接
                return true;
            }
        }
        false
    }

    //给定一个神经细胞ID时，本函数就能找到它在neurons中的位置
    fn get_element_pos(&self, neuron_id: i32) -> usize {
        for i in 0..self.neurons.len() {
            if self.neurons[i].id == neuron_id {
                return i;
            }
        }
        panic!("Genome::get_element_pos() Error!");
    }

    //测试传入的ID是否与已存在得某个神经细胞ID相同
    fn alerady_have_this_neuron_id(&self, id: i32) -> bool {
        for n in &self.neurons {
            if id == n.id {
                return true;
            }
        }
        false
    }
}

impl Ord for Genome {
    fn cmp(&self, other: &Genome) -> Ordering {
        self.fitness.partial_cmp(&other.fitness).unwrap()
    }
}

//比较运算符重载
impl PartialOrd for Genome {
    fn partial_cmp(&self, other: &Genome) -> Option<Ordering> {
        self.fitness.partial_cmp(&other.fitness)
    }
}
//等于运算符重载
impl PartialEq for Genome {
    fn eq(&self, other: &Genome) -> bool {
        self.fitness == other.fitness
    }
}
impl Eq for Genome {}
