use super::genes::Genome;
use super::params;
use super::utils::random_usize;
use std::cmp::Ordering;
//持有给定物种的所有基因组
pub struct Species {
    //保留这个物种的第一个成员的本地副本
    leader: Genome,
    //指向此物种内所有基因组的指针
    members: Vec<usize>,
    //物种需要一个标识号
    species_id: i32,
    //这个物种到目前为止找到最好的适应性分
    best_fitness: f64,
    //几代，因为适应性分已经改善，我们可以使用这个信息来杀死一个物种，如果需要的话
    gens_no_improvement: i32,
    age: i32,
    //这一物种的下一代群体必须孵化出来多少后代
    spawns_rqd: f64,
}

impl Species {
    //这个ctor创建一个新物种的实例。 初始化基因组的本地拷贝保存在self.eader中，self.members的第一个元素是指向该基因组的指针。
    pub fn new(first_org: &Genome, first_org_idx: usize, species_id: i32) -> Species {
        Species {
            species_id: species_id,
            best_fitness: first_org.fitness(),
            gens_no_improvement: 0,
            age: 0,
            leader: first_org.clone(),
            members: vec![first_org_idx],
            spawns_rqd: 0.0,
        }
    }

    //这个函数向这个物种添加一个新的成员，并相应地更新成员变量
    pub fn add_member(&mut self, new_member: &Genome, new_member_idx: usize) {
        //新成员的适应分比最好的适应分更好吗？
        if new_member.fitness() > self.best_fitness {
            self.best_fitness = new_member.fitness();
            self.gens_no_improvement = 0;
            self.leader = new_member.clone();
        }
        self.members.push(new_member_idx);
    }

    //这个功能清除上一代的所有成员，更新年龄和发型没有改善。
    pub fn purge(&mut self) {
        self.members.clear();
        //更新年龄等
        self.age += 1;
        self.gens_no_improvement += 1;
        self.spawns_rqd = 0.0;
    }

    //这一种方法可以增加年轻基因组的适应性分数，惩罚年老基因组的适应性分数，
    //然后对该物种的所有成员实行适应性分数共享
    pub fn adjust_fitnesses(&mut self, genomes: &mut Vec<Genome>) {
        //let mut total = 0.0;
        let len = self.members.len();
        for gen in &self.members {
            let mut fitness = genomes[*gen].fitness();
            //如果物种年轻，提高适应分成绩
            if self.age < params::YOUNG_BONUS_AGE_THRESHHOLD {
                fitness *= params::YOUNG_FITNESS_BONUS;
            }
            //惩罚老物种
            if self.age > params::OLD_AGE_THRESHOLD {
                fitness *= params::OLD_AGE_PENALTY;
            }

            //total += fitness;
            //应用将适应分分享,来调整适应分
            let adjust_fitnesses = fitness / len as f64;
            genomes[*gen].set_adj_fitness(adjust_fitnesses);
        }
    }

    //简单地加上物种中每个人的预期产卵量，以计算这个物种应该产生的后代的数量
    pub fn calculate_spawn_amount(&mut self, genomes: &Vec<Genome>) {
        for gen in &self.members {
            self.spawns_rqd += genomes[*gen].amount_to_spawn();
        }
    }

    //从最佳Params.SurvivalRate百分比中随机选出的物种产生一个个体
    pub fn spawn(&self, genomes: &Vec<Genome>) -> Genome {
        if self.members.len() == 1 {
            return genomes[self.members[0]].clone();
        } else {
            let max_index_size = (params::SURVIVAL_RATE * self.members.len() as f64) as usize + 1;
            let the_one = random_usize(0, max_index_size);
            return genomes[self.members[the_one]].clone();
        }
    }

    pub fn leader(&self) -> &Genome {
        &self.leader
    }
    pub fn num_to_spawn(&self) -> f64 {
        self.spawns_rqd
    }
    pub fn num_members(&self) -> usize {
        self.members.len()
    }
    pub fn gens_no_improvement(&self) -> i32 {
        self.gens_no_improvement
    }
    pub fn id(&self) -> i32 {
        self.species_id
    }
    pub fn _species_leader_fitness(&self) -> f64 {
        self.leader.fitness()
    }
    pub fn best_fitness(&self) -> f64 {
        self.best_fitness
    }
}

impl Ord for Species {
    fn cmp(&self, other: &Species) -> Ordering {
        self.best_fitness.partial_cmp(&other.best_fitness).unwrap()
    }
}

//比较运算符重载
impl PartialOrd for Species {
    fn partial_cmp(&self, other: &Species) -> Option<Ordering> {
        self.best_fitness.partial_cmp(&other.best_fitness)
    }
}
//等于运算符重载
impl PartialEq for Species {
    fn eq(&self, other: &Species) -> bool {
        self.best_fitness == other.best_fitness
    }
}
impl Eq for Species {}
