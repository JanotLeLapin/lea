import static org.junit.Assert.assertEquals;

import org.junit.Test;

public class Main {
  @Test public void return_statement() {
    assertEquals("bar", Tests.basic_return());
    assertEquals("baz", Tests.complex_return());
    assertEquals(5, Tests.num_return());
    assertEquals(false, Tests.bool_return());
  }
}
