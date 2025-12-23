# bound = 100000000
# print(bound)
# sum = 0
# for i in range(bound):
#     sum += i;


def add(a, b):
    return a + b

sum = 0
for i in range(100):
    sum = add(sum, i)

print(add("foo", "bar"))



class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            jit_merge_point(node=self)

            cond = cond_expr.execute_bool(frame)
            if not cond:
                break
            body_expr.execute(frame)


while (cnt < 100) {
    cnt := cnt + 1;
}


class IntLessThan(AstNode):
    child left_expr
    child right_expr

    def execute_bool(frame):
        try:
            left = left_expr.execute_int(frame)
        except UnexpectedResult r:
            ...
        
        try:
            right = right_expr.execute_int(frame)
        except UnexpectedResult r:
            ...

        return left < right


class IntVarRead(AstNode):
    final idx

    def execute_int(frame):
        if frame.is_int(idx):
            return frame.local_int[idx]
        else:
            new_node = respecialize()
            raise UnexpectedResult(new_node)


class IntLiteral(AstNode):
    final value

    def execute_int(frame):
        return value

class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            if not frame.is_int(1):
                __deopt_return_to_interpreter()

            if not (frame.local_int[1] < 100):
                break
            body_expr.execute(frame)


class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            if frame.is_int(1):
                left = frame.local_int[1]
            else:
                __deopt_return_to_interpreter()

            if not (left < 100):
                break
            body_expr.execute(frame)


class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            if frame.is_int(1):
                left = frame.local_int[1]
            else:
                __deopt_return_to_interpreter()
            
            right = 100

            cond = left < right
            if not cond:
                break
            body_expr.execute(frame)

class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            if frame.is_int(1):
                left = frame.local_int[1]
            else:
                __deopt_return_to_interpreter()
            
            try:
                right = 100
            except UnexpectedResult r:
                ...

            cond = left < right
            if not cond:
                break
            body_expr.execute(frame)


class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            if frame.is_int(1):
                left = frame.local_int[1]
            else:
                __deopt_return_to_interpreter()
            
            try:
                right = cond_expr.right_expr.execute_int(frame)
            except UnexpectedResult r:
                ...

            cond = left < right
            if not cond:
                break
            body_expr.execute(frame)

class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            try:
                if frame.is_int(1):
                    left = frame.local_int[1]
                else:
                    new_node = respecialize()
                    raise UnexpectedResult(new_node)
            except UnexpectedResult r:
                ...
            
            try:
                right = cond_expr.right_expr.execute_int(frame)
            except UnexpectedResult r:
                ...

            cond = left < right
            if not cond:
                break
            body_expr.execute(frame)


class WhileNode(AstNode):
    child cond_expr
    child body_expr

    def execute(frame):
        while True:
            try:
                left = cond_expr.left_expr.execute_int(frame)
            except UnexpectedResult r:
                ...
            
            try:
                right = cond_expr.right_expr.execute_int(frame)
            except UnexpectedResult r:
                ...

            cond = left < right
            if not cond:
                break
            body_expr.execute(frame)