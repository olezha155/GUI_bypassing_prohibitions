pub fn merge_sort<T: Ord + Clone>(slice: &mut [T]) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    let mid = len / 2;

    // рекурсивно сортируем левую и правую части
    merge_sort(&mut slice[..mid]);
    merge_sort(&mut slice[mid..]);

    // сливаем отсортированные части
    merge(slice, mid);
}

fn merge<T: Ord + Clone>(slice: &mut [T], mid: usize) {
    let left = slice[..mid].to_vec();
    let right = slice[mid..].to_vec();

    let mut l = 0; // индекс для левой части
    let mut r = 0; // индекс для правой части
    let mut i = 0; // индекс для основного слайса

    while l < left.len() && r < right.len() {
        if left[l] <= right[r] {
            slice[i] = left[l].clone();
            l += 1;
        } else {
            slice[i] = right[r].clone();
            r += 1;
        }
        i += 1;
    }

    // добавляем оставшиеся элементы, если они есть
    if l < left.len() {
        slice[i..].clone_from_slice(&left[l..]);
    }
    if r < right.len() {
        slice[i..].clone_from_slice(&right[r..]);
    }
}
