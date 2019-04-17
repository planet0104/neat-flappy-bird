use super::genes::{Genome, Innovation, LinkGene, NeuronGene};
use super::params;
use super::phenotype::NeuralNet;
use super::species::Species;
use super::utils::{random_float, random_int, random_usize};

// Desc：用于实现的进化算法类
pub struct GA {
    genomes: Vec<Genome>,
    //保留最后一代最佳基因组的记录。 （用于在用户按下“B”键时将显示效果最佳）
    best_genomes: Vec<Genome>,
    species: Vec<Species>,
    innovation: Innovation,
    generation: i32,
    next_genome_id: i32,
    next_species_id: i32,
    pop_size: i32,
    //调整后的适应分
    tot_fit_adj: f64,
    av_fit_adj: f64,
    //指向适合基因组的基因组
    _fittest_genome: i32,
    best_ever_fitness: f64,
    average_fitness: f64,
    //这是预先计算的分割深度。 它们用于计算渲染的神经元x/y位置，并用于计算当“表现型”工作在“快照”模式时网络的刷新深度。
    splits: Vec<SplitDepth>,
    fitness_scores: Vec<f64>, //用来临时存储人口的适应分
}

#[derive(Debug, PartialEq)]
enum ParentType {
    Mum,
    Dad,
}

//该结构用于创建网络深度查找表。
struct SplitDepth {
    val: f64,
    depth: i32,
}
impl SplitDepth {
    pub fn new(v: f64, d: i32) -> SplitDepth {
        SplitDepth { val: v, depth: d }
    }
}

impl GA {
    //这个构造函数从提供的值创建基本基因组，并创建一个“大小”类似（相同拓扑，不同权重）基因组的群体
    pub fn new(size: i32, inputs: usize, outputs: usize) -> GA {
        //创建基因组群体
        let mut next_genome_id = 0;
        let mut genomes: Vec<Genome> = vec![];
        for _ in 0..size {
            genomes.push(Genome::minimal(next_genome_id, inputs, outputs));
            next_genome_id += 1;
        }
        //创建创新列表。 首先创建一个最小的基因组
        let genome = Genome::minimal(1, inputs, outputs);
        //创建创新
        let innovation = Innovation::new(genome.genes(), genome.neurons());
        let mut splits: Vec<SplitDepth> = vec![];
        GA::split(&mut splits, 0.0, 1.0, 0);

        GA {
            pop_size: size,
            generation: 0,
            innovation: innovation,
            next_genome_id: next_genome_id,
            next_species_id: 0,
            _fittest_genome: 0,
            best_ever_fitness: 0.0,
            tot_fit_adj: 0.0,
            av_fit_adj: 0.0,
            best_genomes: vec![],
            genomes: genomes,
            species: vec![],
            splits: splits,
            fitness_scores: vec![0.0; size as usize],
            average_fitness: 0.0,
        }
    }

    pub fn epoch(&mut self) {
        // if self.generation % 500 == 0 {
        //     println!("第{}代", self.generation);
        // }
        //首先进行检测，以保证有正确数量的适应性分数
        if self.fitness_scores.len() != self.genomes.len() {
            panic!("GA::Epoch(适应分/染色体 数目不等)!");
        }
        self.reset_and_kill();
        //用上一次运行获得的适应性分数来更新基因组
        for gen in 0..self.genomes.len() {
            self.genomes[gen].set_fitness(self.fitness_scores[gen]);
        }
        //计算平均分
        let mut total_score = 0.0;
        for score in &self.fitness_scores {
            total_score += *score;
        }
        self.average_fitness = total_score / self.pop_size as f64;
        //对基因组排序，并为最好的表演者作一记录
        self.sort_and_record();
        //把群体分成具有相似拓扑的各种物种，调整他们的适应性分数，并计算孵化级别
        self.speciate_and_calculate_spawn_levels();

        //这将用来保存基因组的新群体
        let mut new_pop: Vec<Genome> = vec![];
        //从每一个物种产生子代。待孵化的自带数目是一个双精度实数，需要将它转换成为一个整数
        let mut num_spawned_so_far = 0;
        //通过对每个物种的循环，选择要配对杂交和突变的子代
        let mut baby = Genome::new();
        for spc in 0..self.species.len() {
            //从每个物种得到的孵化总数是一个浮点数，需要四舍五入化为整数
            //而这有可能导致孵化总数的溢出。本语句确保不会出现这种情况
            if num_spawned_so_far < self.pop_size {
                //这是该物种要求孵化下一代的个体总数,round()把浮点数改大或改小化成整数
                let mut num_to_spawn = self.species[spc].num_to_spawn().round() as i32;
                let mut chosen_bestyet = false;
                while num_to_spawn > 0 {
                    num_to_spawn -= 1;
                    //首先从该物种找出表现最好的基因组，并将它不做任何变异地转移到新的群体
                    //从而为每个物种提供了精英
                    if !chosen_bestyet {
                        baby = self.species[spc].leader().clone();
                        chosen_bestyet = true;
                    } else {
                        //如果本物种仅包含一个个体，则只能执行突变操作
                        if self.species[spc].num_members() == 1 {
                            //孵化一个后代
                            baby = self.species[spc].spawn(&self.genomes);
                        } else {
                            //如果大于1，则可以使用杂交操作
                            //孵化1
                            let g1 = self.species[spc].spawn(&self.genomes);
                            if random_float() < params::CROSSOVER_RATE {
                                //孵化2, 保证它不是g1
                                let mut g2 = self.species[spc].spawn(&self.genomes);
                                let mut num_attempts = 5;
                                while g1.id() == g2.id() && num_attempts > 0 {
                                    num_attempts -= 1;
                                    g2 = self.species[spc].spawn(&self.genomes);
                                }
                                if g1.id() != g2.id() {
                                    baby = self.crossover(&g1, &g2);
                                } else {
                                    baby = g1;
                                }
                            }
                        }

                        self.next_genome_id += 1;
                        baby.set_id(self.next_genome_id);

                        //已存在一个孵化出来的子代，对它进行突变。
                        //首先应考虑加入一个神经细胞的几率
                        if baby.num_neurons() < params::MAX_PERMITTED_NEURONS {
                            baby.add_neuron(
                                params::CHANCE_ADD_NODE,
                                &mut self.innovation,
                                params::NUM_TRYS_TO_FIND_LOOPED_LINK,
                            );
                        }
                        //加入链接的几率
                        baby.add_link(
                            params::CHANCE_ADD_LINK,
                            params::CHANCE_ADD_RECURRENT_LINK,
                            &mut self.innovation,
                            params::NUM_TRYS_TO_FIND_LOOPED_LINK,
                            params::NUM_ADD_LINK_ATTEMPTS,
                        );
                        //对权重实行突变
                        baby.mutate_weights(
                            params::MUTATION_RATE,
                            params::PROBABILITY_WEIGHT_REPLACED,
                            params::MAX_WEIGHT_PERTURBATION,
                        );
                        //对激励响应实行突变
                        baby.mutate_activation_response(
                            params::ACTIVATION_MUTATION_RATE,
                            params::MAX_ACTIVATION_PERTURBATION,
                        );
                    }
                    //根据创新号对新生基因排序
                    baby.sort_genes();

                    //将新基因加入到新群体
                    new_pop.push(baby.clone());
                    num_spawned_so_far += 1;
                    if num_spawned_so_far == self.pop_size {
                        num_to_spawn = 0;
                    }
                } //结束while循环
            } //结束if语句
        } //下一个物种
          //如果这时因舍入误差而使得所有物种孵化总量加起来出现下溢，并使子代总数小于
          //群体的规模，则必须创建附加的子代并加入到新群体。这可以应用锦标赛方式，从群体
          //的所有个体中选择得到
        if num_spawned_so_far < self.pop_size {
            //计算要求增加的子代数目
            let mut rqd = self.pop_size - num_spawned_so_far;
            //捕捉它们
            while rqd > 0 {
                new_pop.push(self.tournament_selection(self.pop_size / 5));
                rqd -= 1;
            }
        }

        //用新群体替代当前群体
        self.genomes = new_pop;
        //创建新的表现型
        // for gen in 0..self.genomes.len() {
        //     //计算最大网络深度
        //     let depth = self.calculate_net_depth(&self.genomes[gen]);
        //     self.genomes[gen].create_phenotype(depth);
        // }
        self.create_phenotypes();
        //增加代数计数器
        self.generation += 1;
    }

    //遍历人口的所有成员，并创建他们的表型。 返回一个包含指向新表型的指针的向量
    pub fn create_phenotypes(&mut self) {
        for i in 0..self.pop_size as usize {
            //计算最大网络深度
            let depth = self.calculate_net_depth(&self.genomes[i]);
            self.genomes[i].create_phenotype(depth);
        }
    }

    pub fn get_phenotype(&mut self, index: usize) -> &mut NeuralNet {
        self.genomes[index].phenotype()
    }

    pub fn fitness_scores(&mut self) -> &mut Vec<f64> {
        &mut self.fitness_scores
    }

    //这个函数简单地遍历每个物种，并调用每个物种的AdjustFitness
    fn adjust_species_fitnesses(&mut self) {
        for sp in &mut self.species {
            sp.adjust_fitnesses(&mut self.genomes);
        }
    }

    //在查找表中搜索基因组中每个节点的dSplitY值，并根据该图返回网络的深度
    fn calculate_net_depth(&self, gen: &Genome) -> i32 {
        let mut max_so_far = 0;
        for nd in 0..gen.num_neurons() {
            for sp in &self.splits {
                if gen.split_y(nd) == sp.val && sp.depth > max_so_far {
                    max_so_far = sp.depth;
                }
            }
        }
        max_so_far + 2
    }

    /** 锦标赛选择 */
    fn tournament_selection(&self, num_comparisons: i32) -> Genome {
        let mut best_fitness_so_far = 0.0;
        let mut chosen_one = 0;
        //从人群中选择 num_comparisons 成员进行随机测试，达到目前为止最好的发现
        for _ in 0..num_comparisons {
            let this_try = random_usize(0, self.genomes.len() - 1);
            if self.genomes[this_try].fitness() > best_fitness_so_far {
                chosen_one = this_try;
                best_fitness_so_far = self.genomes[this_try].fitness();
            }
        }
        //返回冠军
        self.genomes[chosen_one].clone()
    }

    pub fn crossover(&mut self, mum: &Genome, dad: &Genome) -> Genome {
        //首先计算用来产生disjoint/excess基因的基因组。这是适应性最好的基因组。
        //如果他们有相同的适应分，则选用较短者(因为希望网络保持尽可能小)
        let best = if mum.fitness() == dad.fitness() {
            //如果他们有相同的适应性又有相同的长度，则按随机方式来选择一个
            if mum.num_genes() == dad.num_genes() {
                match random_int(0, 1) {
                    0 | 1 => ParentType::Mum,
                    _ => ParentType::Mum,
                }
            } else {
                if mum.num_genes() < dad.num_genes() {
                    ParentType::Mum
                } else {
                    ParentType::Dad
                }
            }
        } else {
            if mum.fitness() > dad.fitness() {
                ParentType::Mum
            } else {
                ParentType::Dad
            }
        };

        //这些向量保存了子代神经细胞和基因
        let mut baby_neurons: Vec<NeuronGene> = vec![];
        let mut baby_genes: Vec<LinkGene> = vec![];
        //用于存放所有被加入神经细胞的标识号的临时vector
        let mut neurons: Vec<i32> = vec![];
        //创建两个迭代变量，这样可以一步步通过每一个父代(Mum,Dad)基因，并把两个迭代变量设置为每一父代的第一对基因
        let mut iter_mum = mum.genes().into_iter();
        let mut iter_dad = dad.genes().into_iter();
        let mut selected_gene: Option<&LinkGene> = None;
        let mut cur_mum = iter_mum.next();
        let mut cur_dad = iter_dad.next();

        while !(cur_mum == None && cur_dad == None) {
            //妈妈基因的结尾已经到达
            if cur_mum == None && cur_dad != None {
                if best == ParentType::Dad {
                    //加入爸爸的基因
                    selected_gene = cur_dad;
                }
                //考察爸爸的下一个基因
                cur_dad = iter_dad.next();
            }
            //已经达到爸爸基因的结尾
            else if cur_dad == None && cur_mum != None {
                //如果妈妈最适应
                if best == ParentType::Mum {
                    //加入妈妈的基因
                    selected_gene = cur_mum;
                }
                //移动到妈妈的下一个基因
                cur_mum = iter_mum.next();
            }
            //如果妈妈的创新标识小于爸爸的创新数标识
            else if cur_mum.unwrap().innovation_id() < cur_dad.unwrap().innovation_id() {
                //如果妈妈是最适应者，则加入妈妈基因
                if best == ParentType::Mum {
                    selected_gene = cur_mum;
                }
                //移动到妈妈的下一基因
                cur_mum = iter_mum.next();
            }
            //如果爸爸的创新号小于妈妈的创新号
            else if cur_dad.unwrap().innovation_id() < cur_mum.unwrap().innovation_id() {
                //如果爸爸是最适应者，则加入爸爸基因
                if best == ParentType::Dad {
                    selected_gene = cur_dad;
                }
                //移动到妈妈的下一基因
                cur_dad = iter_dad.next();
            }
            //如果爸爸妈妈的创新号一样
            else if cur_dad.unwrap().innovation_id() == cur_mum.unwrap().innovation_id() {
                //爸爸、妈妈二者都取出基因
                if random_float() < 0.5 {
                    selected_gene = cur_mum;
                } else {
                    selected_gene = cur_dad;
                }
                //移动到妈妈的下一基因
                cur_dad = iter_dad.next();
                cur_mum = iter_mum.next();
            }
            let selected_gene = selected_gene.unwrap();
            //如果原来未曾加入所选择的基因, 则现在将它加入
            if baby_genes.len() == 0 {
                baby_genes.push((*selected_gene).clone());
            } else {
                if baby_genes[baby_genes.len() - 1].innovation_id() != selected_gene.innovation_id()
                {
                    baby_genes.push((*selected_gene).clone());
                }
            }
            //检查neurons是否已经有所选基因selected_gene所涉及(关联)的神经细胞?
            //如果没有，就需要(在neurons中)加入这些神经细胞
            GA::add_neuron_id(selected_gene.from_neuron(), &mut neurons);
            GA::add_neuron_id(selected_gene.to_neuron(), &mut neurons);
        } //结束while循环
          //创建所要的全部神经细胞，首先将它们排序
          //neurons.sort();// 正序
        neurons.sort_by(|a, b| b.cmp(a)); //从大到小排序
        for neuron_id in neurons {
            baby_neurons.push(self.innovation.create_neuron_from_id(neuron_id));
        }
        //最后创建基因
        let baby_genome = Genome::from(
            self.next_genome_id,
            baby_neurons,
            baby_genes,
            mum.num_inputs(),
            mum.num_outputs(),
        );
        self.next_genome_id += 1;

        baby_genome
    }

    //只是检查一下节点ID是否已经被添加到节点的向量中。 如果没有，则添加新的ID。 用于杂交
    fn add_neuron_id(node_id: i32, vec: &mut Vec<i32>) {
        for i in 0..vec.len() {
            if vec[i] == node_id {
                //已经有了
                return;
            }
        }
        vec.push(node_id);
    }

    //这个功能可以重置一些准备好下一个纪元的值，杀死所有的表型和任何表现不佳的物种。
    fn reset_and_kill(&mut self) {
        self.tot_fit_adj = 0.0;
        self.av_fit_adj = 0.0;

        let best_ever_fitness = self.best_ever_fitness;
        //println!("reset_and_kill>self.species.len()>>01>>{}", self.species.len());
        for sp in &mut self.species {
            sp.purge();
        }
        //清除物种
        self.species.retain(|cur_sp| {
            //如果没有改善或者不是迄今为止发现了最好的基因组的物种，杀死物种
            !(cur_sp.gens_no_improvement() > params::NUM_GENS_ALLOWED_NO_IMPROVEMENT
                && cur_sp.best_fitness() < best_ever_fitness)
        });
        //println!("reset_and_kill>self.species.len()>>02>>{}", self.species.len());
        //我们也可以删除表型
        for gen in &mut self.genomes {
            gen.delete_phenotype();
        }
    }

    //将人口排序为降序适应分，保留最佳n基因组的记录，并相应地更新任何适应分统计数据
    fn sort_and_record(&mut self) {
        //根据未经调整（无适应分分享）适应度的方式对基因组进行排序
        //self.genomes.sort();
        self.genomes.sort_by(|a, b| b.cmp(a)); //从大到小排序
                                               //是这一代最好的基因组吗？
        if self.genomes[0].fitness() > self.best_ever_fitness {
            self.best_ever_fitness = self.genomes[0].fitness();
        }
        //保存最好的基因组的记录
        self.store_best_genomes();
    }

    //用于保留以前种群最佳基因组的记录，以便在需要时可以显示它们。
    pub fn store_best_genomes(&mut self) {
        //删除旧的记录
        self.best_genomes.clear();
        for gen in 0..params::NUM_BEST_SPRITE {
            self.best_genomes.push(self.genomes[gen].clone());
        }
    }

    //返回上一代n个最佳表型的Vec<index:i32>
    pub fn get_best_phenotypes_from_last_generation(&mut self) -> Vec<usize> {
        let mut brains: Vec<usize> = vec![];
        for gen in 0..self.best_genomes.len() {
            //计算最大网络深度
            let depth = self.calculate_net_depth(&self.best_genomes[gen]);
            self.best_genomes[gen].create_phenotype(depth);
            brains.push(gen);
        }
        brains
    }

    //通过与人口中的每个其他成员计算兼容性分数，并相应地确定每个个体与其各自的种类。
    //然后，该功能根据物种年龄和分享情况调整每个人的适应度，
    //并确定每个人应该产生多少个后代。
    fn speciate_and_calculate_spawn_levels(&mut self) {
        //遍历每个基因组并指定
        for gen in 0..self.genomes.len() {
            //计算其与每个物种领导者的兼容性得分。 兼容添加物种。
            //如果没有，创造一个新的物种
            let mut spc_selected = -1;
            for spc in 0..self.species.len() {
                let campatibility =
                    self.genomes[gen].get_compatibility_score(self.species[spc].leader());
                //如果这个体类似于这个物种添加到物种
                if campatibility <= params::COMPATIBILITY_THRESHOLD {
                    spc_selected = spc as i32;
                    break;
                }
            }
            if spc_selected != -1 {
                self.species[spc_selected as usize].add_member(&self.genomes[gen], gen);
                self.genomes[gen].set_species(self.species[spc_selected as usize].id());
            } else {
                //我们没有找到兼容的物种，所以让我们创建一个新的物种
                self.species
                    .push(Species::new(&self.genomes[gen], gen, self.next_species_id));
                self.next_species_id += 1;
            }
        }

        //现在所有的基因组都被分配了一个物种，需要调整适应度分数以考虑分享和物种年龄。
        //把群体分成具有相似拓扑的各种物种,调整他们的适应性分数,并计算孵化级别
        self.adjust_species_fitnesses();

        //计算人口的新调整总和平均适应度
        for gen in &self.genomes {
            self.tot_fit_adj += gen.get_adj_fitness();
        }
        self.av_fit_adj = self.tot_fit_adj / self.genomes.len() as f64;

        //计算每个成员人数应该产生多少个后代
        for gen in &mut self.genomes {
            let to_spawn = gen.get_adj_fitness() / self.av_fit_adj;
            gen.set_amount_to_spawn(to_spawn);
        }

        //迭代所有的物种，并计算每个物种应该产生多少个后代
        for spc in &mut self.species {
            spc.calculate_spawn_amount(&self.genomes);
        }
    }

    //该函数用于创建一个用于计算网络深度的查找表。
    fn split(splits: &mut Vec<SplitDepth>, low: f64, high: f64, depth: i32) {
        let span = high - low;
        splits.push(SplitDepth::new(low + span / 2.0, depth + 1));
        if depth <= 6 {
            GA::split(splits, low, low + span / 2.0, depth + 1);
            GA::split(splits, low + span / 2.0, high, depth + 1);
        }
    }

    pub fn num_species(&self) -> i32 {
        self.species.len() as i32
    }

    pub fn num_genomes(&self) -> i32 {
        self.genomes.len() as i32
    }

    pub fn num_best_genomes(&self) -> i32 {
        self.best_genomes.len() as i32
    }

    pub fn best_ever_fitness(&self) -> f64 {
        self.best_ever_fitness
    }

    pub fn average_fitness(&self) -> f64 {
        self.average_fitness
    }

    pub fn pop_size(&self) -> i32 {
        self.pop_size
    }

    // pub fn render_species_info(
    //     &self,
    //     surface: ui::Surface,
    //     mut left: i32,
    //     top: i32,
    //     right: i32,
    //     bottom: i32,
    // ) {
    //     if self.species.len() < 1 {
    //         return;
    //     }
    //     let num_colours = 255 / self.species.len() as i32;
    //     let slice_per_sprite = (right - left) as f64 / (self.pop_size - 1) as f64;

    //     //现在为每个物种绘制一个不同的彩色矩形
    //     for spc in 0..self.species.len() {
    //         //选择画刷
    //         let color = ui::rgb(
    //             num_colours * spc as i32,
    //             255,
    //             255 - num_colours * spc as i32,
    //         );
    //         if spc == self.species.len() - 1 {
    //             ui::rectangle(surface, left, top, right, bottom, color);
    //         } else {
    //             ui::rectangle(
    //                 surface,
    //                 left,
    //                 top,
    //                 (left as f64 + slice_per_sprite * self.species[spc].num_members() as f64)
    //                     as i32,
    //                 bottom,
    //                 color,
    //             );
    //         }

    //         left += (slice_per_sprite * self.species[spc].num_members() as f64) as i32;

    //         //显示最佳表现物种统计信息
    //         if self.species[spc].best_fitness() == self.best_ever_fitness {
    //             let mut s = format!("最好物种ID: {}", self.species[spc].id());
    //             ui::text_out(surface, 5, top - 80, &s);

    //             s = format!("物种年龄: {}", self.species[spc].age());
    //             ui::text_out(surface, 5, top - 60, &s);

    //             s = format!(
    //                 "无改善基因数: {}",
    //                 self.species[spc].gens_no_improvement()
    //             );
    //             ui::text_out(surface, 5, top - 40, &s);
    //         }

    //         ui::text_out(surface, 5, top - 20, &"物种分配栏:");
    //     }
    // }
}
