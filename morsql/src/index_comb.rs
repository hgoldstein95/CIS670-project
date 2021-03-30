#![allow(dead_code)]


#[derive(Debug)]
pub struct IndexCombinations{
    pub bounds : Vec<usize>,
    pub indices : Vec<usize>,
    pub started : bool
}


pub fn given_bounds(bounds : Vec<usize>) -> IndexCombinations {
    let mut indices = Vec::with_capacity(bounds.len());
    for _i in 0..(bounds.len()){
        indices.push(0);
    }
    return IndexCombinations {
        bounds : bounds,
        indices : indices,
        started : false
    };
}


impl Iterator for IndexCombinations{
    type Item = Vec<usize>;
    //treats the list of indices like a number whose least significant digit is at index 0
    fn next(&mut self) -> Option<Vec<usize>>{
        if !self.started {
            self.started = true;
            return Some (self.indices.to_vec());
        }
        let mut carry_in = true;
        for i in 0 .. (self.indices.len()){
            if self.indices[i] == self.bounds[i] - 1 {
                self.indices[i] = 0;
            } else {
                self.indices[i] += 1;
                carry_in = false;
                break;
            }
        }
        if carry_in {
            return None;
        } else {
            return Some(self.indices.to_vec());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::index_comb::given_bounds;

    #[test]
    fn index_test1() {
        let bounds = vec![2,2,2];
        let mut iter = given_bounds(bounds);
        let i0 = iter.next();
        assert_eq!(i0, Some (vec![0,0,0]));
        let i1 = iter.next();
        assert_eq!(i1, Some (vec![1,0,0]));
        let i2 = iter.next();
        assert_eq!(i2, Some (vec![0,1,0]));
        let i3 = iter.next();
        assert_eq!(i3, Some (vec![1,1,0]));
        let i4 = iter.next();
        assert_eq!(i4, Some (vec![0,0,1]));
        let i5 = iter.next();
        assert_eq!(i5, Some (vec![1,0,1]));
        let i6 = iter.next();
        assert_eq!(i6, Some (vec![0,1,1]));
        let i7 = iter.next();
        assert_eq!(i7, Some (vec![1,1,1]));
        let i8 = iter.next();
        assert_eq!(i8, None);
    }
}